//! Generic Arcadia scheduling subsystem.
//!
//! Worker pool, interval/delay timers, and a main-thread queue drained by the GUI run loop.
//! Used by `tray`, `cursor`, Python `arcadia.set_timer`, and any other module that needs
//! deferred or periodic work without dragging in a full async runtime.
//!
//! ## Runtime model
//!
//! - **Timer thread** — holds a min-heap of pending fires; enqueues work to the pool (never runs
//!   slow callbacks itself).
//! - **Worker pool** — fixed set of `arcadia-worker-*` threads plus one **`arcadia-python-timer`**
//!   thread for [`TaskLane::PythonTimer`] jobs. Timer fires and [`spawn`] submit jobs here instead
//!   of spawning unbounded OS threads.
//! - **Lanes** — [`TaskLane::Default`] and [`TaskLane::Blocking`] share the main workers; each
//!   worker drains the default queue first, then the blocking queue, so short work stays
//!   responsive when blocking jobs are queued. [`TaskLane::PythonTimer`] uses a dedicated thread
//!   so extension timer callbacks (GIL) serialize without contending with the general pool.
//! - **Backpressure** — job queues are **unbounded**; producers never drop jobs. Memory can grow
//!   if producers outpace consumers (same class as unbounded `thread::spawn` backlog, but without
//!   thread explosion).
//! - **Cancellation** — [`cancel`] notifies the timer thread and marks the task id in a shared set;
//!   workers skip jobs whose id was cancelled before execution. Best-effort: a job already running
//!   is not interrupted.
//!
//! In headless mode [`run_on_main`] runs inline because there is no separate UI thread to dispatch to.
//!
//! ## AI / long-running work
//!
//! Use [`spawn_blocking`] for CPU- or IO-heavy work. Post results to the UI with [`run_on_main`]
//! (never block the main thread waiting on the pool without a timeout — risk of deadlock if a worker
//! needs the main queue to drain).
//!
//! ## Observability (Phase 3)
//!
//! - **[`pool_stats`]** — completed job count, current active jobs, peak concurrency.
//! - **Slow job log** — set `ARCADIA_SCHED_SLOW_MS` to a positive millisecond threshold; jobs
//!   exceeding it log one line to **stderr** (`eprintln!`, no extra crate dependency).
//! - **Shutdown** — [`request_scheduler_shutdown`] wakes workers so they exit; [`join_scheduler_workers`]
//!   joins pool threads (see that function’s **safety** note — do not use on the global pool during
//!   normal `cargo test` unless you accept a dead scheduler for later tests).

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering as AOrdering};
use std::sync::{mpsc, Arc, Condvar, Mutex, OnceLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub type TaskId = u64;

/// Snapshot from [`pool_stats`] for diagnostics or tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SchedPoolStats {
    /// Pool jobs that finished execution (not counting jobs skipped due to cancel before run).
    pub jobs_completed: u64,
    /// Jobs currently executing on worker threads.
    pub jobs_active: usize,
    /// High-water mark of concurrent pool jobs since process start.
    pub peak_concurrent_jobs: usize,
}

/// Read [`SchedPoolStats`] from the process-global scheduler pool.
pub fn pool_stats() -> SchedPoolStats {
    let p = &scheduler().pool;
    SchedPoolStats {
        jobs_completed: p.jobs_completed.load(AOrdering::Relaxed),
        jobs_active: p.active_jobs.load(AOrdering::Relaxed) as usize,
        peak_concurrent_jobs: p.peak_active_jobs.load(AOrdering::Relaxed) as usize,
    }
}

/// Request worker threads to exit after their current wait point. Wakes blocked workers.
///
/// Does **not** stop the timer thread or drain queued jobs; intended for embedders shutting down
/// or tests using a **standalone** pool (see [`join_scheduler_workers`]).
pub fn request_scheduler_shutdown() {
    scheduler().pool.request_shutdown();
}

/// Join all **pool** worker threads after [`request_scheduler_shutdown`].
///
/// # Safety / usage
///
/// The Arcadia process uses a single global pool. Calling this in a unit test **after**
/// `request_scheduler_shutdown` leaves [`spawn`] and timers unusable for the rest of the test
/// process. Prefer testing shutdown with a standalone pool (`WorkPool::new` in this module’s unit
/// tests). Process exit handlers may call shutdown + join for a clean teardown.
pub fn join_scheduler_workers() {
    scheduler().pool.join_workers();
}

fn slow_job_threshold_ms() -> Option<u64> {
    static THRESHOLD_MS: OnceLock<Option<u64>> = OnceLock::new();
    *THRESHOLD_MS.get_or_init(|| {
        std::env::var("ARCADIA_SCHED_SLOW_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|&ms| ms > 0)
    })
}

/// Queue lane for pooled work. [`TaskLane::PythonTimer`] is intended for Python `set_timer` ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskLane {
    /// General fire-and-forget and default timer callbacks.
    Default,
    /// Long-running CPU or blocking IO — same worker set as [`TaskLane::Default`], but dequeued
    /// only when no default jobs are waiting.
    Blocking,
    /// Serialized on `arcadia-python-timer` (one thread).
    PythonTimer,
}

type WorkFn = Box<dyn FnOnce() + Send + 'static>;
type RecurringFn = Arc<dyn Fn() + Send + Sync + 'static>;

struct TimerEntry {
    fire_at: Instant,
    id: TaskId,
    period: Option<Duration>,
    handler: RecurringFn,
    lane: TaskLane,
}

impl PartialEq for TimerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.fire_at == other.fire_at && self.id == other.id
    }
}

impl Eq for TimerEntry {}

impl Ord for TimerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .fire_at
            .cmp(&self.fire_at)
            .then_with(|| other.id.cmp(&self.id))
    }
}

impl PartialOrd for TimerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

enum TimerMsg {
    Schedule(TimerEntry),
    Cancel(TaskId),
    /// Drop every recurring (`spawn_interval`) task from the heap — used when the Python host
    /// reloads so stale `set_timer` callbacks cannot touch tray ids removed in the same pass.
    PurgeRecurring,
}

enum JobKind {
    Once(Box<dyn FnOnce() + Send>),
    Recurring(Arc<dyn Fn() + Send + Sync>),
}

struct Job {
    id: TaskId,
    lane: TaskLane,
    kind: JobKind,
}

struct PoolState {
    default_q: VecDeque<Job>,
    blocking_q: VecDeque<Job>,
    python_q: VecDeque<Job>,
}

struct WorkPool {
    state: Mutex<PoolState>,
    cv: Condvar,
    cancelled: Mutex<HashSet<TaskId>>,
    /// Jobs currently executing on this pool (all lanes).
    active_jobs: AtomicUsize,
    /// High-water mark of `active_jobs` (for tests / diagnostics).
    peak_active_jobs: AtomicUsize,
    /// Jobs that completed execution (cancel-before-run excluded).
    jobs_completed: AtomicU64,
    shutdown: AtomicBool,
    join_handles: Mutex<Vec<JoinHandle<()>>>,
}

impl WorkPool {
    fn new(worker_count: usize) -> Arc<Self> {
        let pool = Arc::new(Self {
            state: Mutex::new(PoolState {
                default_q: VecDeque::new(),
                blocking_q: VecDeque::new(),
                python_q: VecDeque::new(),
            }),
            cv: Condvar::new(),
            cancelled: Mutex::new(HashSet::new()),
            active_jobs: AtomicUsize::new(0),
            peak_active_jobs: AtomicUsize::new(0),
            jobs_completed: AtomicU64::new(0),
            shutdown: AtomicBool::new(false),
            join_handles: Mutex::new(Vec::new()),
        });

        for i in 0..worker_count {
            let p = Arc::clone(&pool);
            let h = thread::Builder::new()
                .name(format!("arcadia-worker-{i}"))
                .spawn(move || p.run_main_worker_loop())
                .expect("arcadia worker thread spawn failed");
            pool.join_handles
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(h);
        }

        let p_py = Arc::clone(&pool);
        let h_py = thread::Builder::new()
            .name("arcadia-python-timer".into())
            .spawn(move || p_py.run_python_worker_loop())
            .expect("arcadia-python-timer thread spawn failed");
        pool.join_handles
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(h_py);

        pool
    }

    fn request_shutdown(&self) {
        self.shutdown.store(true, AOrdering::Release);
        self.cv.notify_all();
    }

    fn join_workers(&self) {
        let handles: Vec<JoinHandle<()>> = {
            let mut g = self.join_handles.lock().unwrap_or_else(|e| e.into_inner());
            std::mem::take(&mut *g)
        };
        for h in handles {
            let _ = h.join();
        }
    }

    fn bump_peak_active(&self) {
        let n = self.active_jobs.fetch_add(1, AOrdering::Relaxed) + 1;
        let mut peak = self.peak_active_jobs.load(AOrdering::Relaxed);
        while n > peak {
            match self.peak_active_jobs.compare_exchange_weak(
                peak,
                n,
                AOrdering::Relaxed,
                AOrdering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
    }

    fn enqueue(&self, job: Job) {
        let lane = job.lane;
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        match lane {
            TaskLane::PythonTimer => st.python_q.push_back(job),
            TaskLane::Default => st.default_q.push_back(job),
            TaskLane::Blocking => st.blocking_q.push_back(job),
        }
        drop(st);
        self.cv.notify_all();
    }

    fn run_main_worker_loop(self: Arc<Self>) {
        loop {
            if self.shutdown.load(AOrdering::Acquire) {
                return;
            }
            let job = {
                let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
                loop {
                    if self.shutdown.load(AOrdering::Acquire) {
                        return;
                    }
                    if let Some(j) = st.default_q.pop_front() {
                        break j;
                    }
                    if let Some(j) = st.blocking_q.pop_front() {
                        break j;
                    }
                    st = self.cv.wait(st).unwrap_or_else(|e| e.into_inner());
                }
            };
            self.run_job(job);
        }
    }

    fn run_python_worker_loop(self: Arc<Self>) {
        loop {
            if self.shutdown.load(AOrdering::Acquire) {
                return;
            }
            let job = {
                let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
                loop {
                    if self.shutdown.load(AOrdering::Acquire) {
                        return;
                    }
                    if let Some(j) = st.python_q.pop_front() {
                        break j;
                    }
                    st = self.cv.wait(st).unwrap_or_else(|e| e.into_inner());
                }
            };
            self.run_job(job);
        }
    }

    fn run_job(&self, job: Job) {
        let was_cancelled = self
            .cancelled
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&job.id);
        if was_cancelled {
            return;
        }

        self.bump_peak_active();
        let id = job.id;
        let lane = job.lane;
        let start = Instant::now();
        match job.kind {
            JobKind::Once(f) => {
                f();
            }
            JobKind::Recurring(arc) => {
                arc();
            }
        }
        let elapsed = start.elapsed();
        self.active_jobs.fetch_sub(1, AOrdering::Relaxed);
        self.jobs_completed.fetch_add(1, AOrdering::Relaxed);
        if let Some(th_ms) = slow_job_threshold_ms() {
            if elapsed.as_millis() > u128::from(th_ms) {
                eprintln!(
                    "arcadia-sched: slow job task_id={id} lane={lane:?} elapsed={elapsed:?} (threshold {}ms from ARCADIA_SCHED_SLOW_MS)",
                    th_ms
                );
            }
        }
    }

    fn mark_cancelled(&self, id: TaskId) {
        let _ = self
            .cancelled
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id);
    }

    /// Drop queued jobs matching `id` (recurring timers can enqueue faster than workers run).
    fn purge_queued_jobs(&self, id: TaskId) {
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        st.default_q.retain(|j| j.id != id);
        st.blocking_q.retain(|j| j.id != id);
        st.python_q.retain(|j| j.id != id);
    }

    /// Drain all queued (not yet started) Python-timer jobs. Called from `cancel_all_recurring`
    /// so that ticks already enqueued before the heap purge cannot fire after tray teardown.
    fn drain_python_queue(&self) {
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        st.python_q.clear();
    }
}

/// Peak number of pool jobs executing at once since process start (test / diagnostics).
#[cfg(test)]
pub(crate) fn test_peak_active_jobs() -> usize {
    scheduler().pool.peak_active_jobs.load(AOrdering::Relaxed)
}

struct Scheduler {
    next_id: AtomicU64,
    timer_tx: mpsc::Sender<TimerMsg>,
    main_queue: Mutex<Vec<WorkFn>>,
    pool: Arc<WorkPool>,
}

static MAIN_DRAINER_REGISTERED: AtomicBool = AtomicBool::new(false);

const MAX_DEFAULT_WORKERS: usize = 8;

fn worker_count() -> usize {
    const ENV_KEY: &str = "ARCADIA_SCHED_WORKERS";
    if let Ok(s) = std::env::var(ENV_KEY) {
        if let Ok(n) = s.parse::<usize>() {
            return n.clamp(1, MAX_DEFAULT_WORKERS);
        }
    }
    thread::available_parallelism()
        .map(|n| n.get().clamp(1, MAX_DEFAULT_WORKERS))
        .unwrap_or(1)
}

fn scheduler() -> &'static Scheduler {
    static SCHEDULER: OnceLock<Scheduler> = OnceLock::new();
    SCHEDULER.get_or_init(|| {
        let pool = WorkPool::new(worker_count());
        let pool_timer = Arc::clone(&pool);
        let (timer_tx, timer_rx) = mpsc::channel::<TimerMsg>();
        thread::Builder::new()
            .name("arcadia-timer".into())
            .spawn(move || run_timer_thread(timer_rx, pool_timer))
            .expect("arcadia-timer thread spawn failed");
        Scheduler {
            next_id: AtomicU64::new(1),
            timer_tx,
            main_queue: Mutex::new(Vec::new()),
            pool,
        }
    })
}

fn run_timer_thread(rx: mpsc::Receiver<TimerMsg>, pool: Arc<WorkPool>) {
    let mut heap: BinaryHeap<TimerEntry> = BinaryHeap::new();
    let mut cancelled: HashSet<TaskId> = HashSet::new();

    loop {
        let wait = heap
            .peek()
            .map(|top| top.fire_at.saturating_duration_since(Instant::now()))
            .unwrap_or_else(|| Duration::from_secs(60));

        match rx.recv_timeout(wait) {
            Ok(TimerMsg::Schedule(entry)) => {
                cancelled.remove(&entry.id);
                heap.push(entry);
            }
            Ok(TimerMsg::Cancel(id)) => {
                cancelled.insert(id);
            }
            Ok(TimerMsg::PurgeRecurring) => {
                let mut kept = BinaryHeap::new();
                while let Some(e) = heap.pop() {
                    if e.period.is_none() {
                        kept.push(e);
                    }
                }
                heap = kept;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }

        let now = Instant::now();
        while let Some(top) = heap.peek() {
            if top.fire_at > now {
                break;
            }
            let entry = heap.pop().expect("peek succeeded");
            if cancelled.contains(&entry.id) {
                continue;
            }
            if pool
                .cancelled
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .contains(&entry.id)
            {
                continue;
            }
            let id = entry.id;
            let lane = entry.lane;
            let handler = Arc::clone(&entry.handler);
            pool.enqueue(Job {
                id,
                lane,
                kind: JobKind::Recurring(handler),
            });
            if let Some(period) = entry.period {
                let still_ok = !cancelled.contains(&entry.id)
                    && !pool
                        .cancelled
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .contains(&entry.id);
                if still_ok {
                    heap.push(TimerEntry {
                        fire_at: now + period,
                        id: entry.id,
                        period: Some(period),
                        handler: entry.handler,
                        lane: entry.lane,
                    });
                }
            }
        }
    }
}

/// Fire-and-forget work on a worker thread ([`TaskLane::Default`]).
pub fn spawn<F: FnOnce() + Send + 'static>(f: F) -> TaskId {
    spawn_on_lane(TaskLane::Default, f)
}

/// Same as [`spawn`] but uses [`TaskLane::Blocking`] (dequeued only when the default queue is empty).
pub fn spawn_blocking<F: FnOnce() + Send + 'static>(f: F) -> TaskId {
    spawn_on_lane(TaskLane::Blocking, f)
}

/// Fire-and-forget work on the pool. [`TaskLane::PythonTimer`] is treated as [`TaskLane::Default`]
/// for one-shots (only recurring timers should use the Python lane).
pub fn spawn_on_lane<F: FnOnce() + Send + 'static>(lane: TaskLane, f: F) -> TaskId {
    let id = scheduler().next_id.fetch_add(1, AOrdering::Relaxed);
    let lane = if matches!(lane, TaskLane::PythonTimer) {
        TaskLane::Default
    } else {
        lane
    };
    let job = Job {
        id,
        lane,
        kind: JobKind::Once(Box::new(f)),
    };
    scheduler().pool.enqueue(job);
    id
}

/// One-shot delayed work.
pub fn spawn_delay<F>(delay: Duration, f: F) -> TaskId
where
    F: FnOnce() + Send + 'static,
{
    let id = scheduler().next_id.fetch_add(1, AOrdering::Relaxed);
    let once_cell: Arc<Mutex<Option<F>>> = Arc::new(Mutex::new(Some(f)));
    let handler: RecurringFn = Arc::new(move || {
        let taken = once_cell.lock().ok().and_then(|mut m| m.take());
        if let Some(g) = taken {
            g();
        }
    });
    let _ = scheduler().timer_tx.send(TimerMsg::Schedule(TimerEntry {
        fire_at: Instant::now() + delay,
        id,
        period: None,
        handler,
        lane: TaskLane::Default,
    }));
    id
}

/// Recurring work fired every `period`. First fire happens after `period`. [`TaskLane::Default`].
pub fn spawn_interval<F>(period: Duration, f: F) -> TaskId
where
    F: Fn() + Send + Sync + 'static,
{
    spawn_interval_on_lane(TaskLane::Default, period, f)
}

/// Recurring work on the given [`TaskLane`] (use [`TaskLane::PythonTimer`] for Python extension timers).
pub fn spawn_interval_on_lane<F>(lane: TaskLane, period: Duration, f: F) -> TaskId
where
    F: Fn() + Send + Sync + 'static,
{
    let id = scheduler().next_id.fetch_add(1, AOrdering::Relaxed);
    let handler: RecurringFn = Arc::new(f);
    let _ = scheduler().timer_tx.send(TimerMsg::Schedule(TimerEntry {
        fire_at: Instant::now() + period,
        id,
        period: Some(period),
        handler,
        lane,
    }));
    id
}

/// Cancel a timer, recurring task, or a queued / not-yet-started pooled job ([`spawn`],
/// [`spawn_blocking`], etc.). Best-effort: work already running is not interrupted. For AI or
/// other async-style flows, treat completion as [`run_on_main`] from the worker after checking
/// cancellation yourself if you need mid-flight abort.
pub fn cancel(id: TaskId) {
    scheduler().pool.mark_cancelled(id);
    let _ = scheduler().timer_tx.send(TimerMsg::Cancel(id));
    scheduler().pool.purge_queued_jobs(id);
}

/// Alias for [`cancel`] (discoverability for “try cancel this task id” call sites).
#[inline]
pub fn try_cancel_task(id: TaskId) {
    cancel(id);
}

/// Remove all **recurring** (`spawn_interval`) tasks. One-shot `spawn_delay` tasks stay scheduled.
///
/// Used when reloading Python extensions so old interval handlers stop before tray teardown.
/// Also drains any Python-timer jobs already enqueued in the pool so they cannot fire after
/// tray items are removed in the same reload pass.
pub fn cancel_all_recurring() {
    let _ = scheduler().timer_tx.send(TimerMsg::PurgeRecurring);
    scheduler().pool.drain_python_queue();
}

/// Queue work to run on the main UI thread. In headless mode runs inline immediately.
pub fn run_on_main<F: FnOnce() + Send + 'static>(f: F) {
    if MAIN_DRAINER_REGISTERED.load(AOrdering::Acquire) {
        if let Ok(mut q) = scheduler().main_queue.lock() {
            q.push(Box::new(f));
        }
    } else {
        f();
    }
}

/// Register that a host (GUI run loop) will periodically call [`drain_main_queue`].
/// Until this is called, [`run_on_main`] runs inline on the caller's thread.
pub fn register_main_thread_drainer() {
    MAIN_DRAINER_REGISTERED.store(true, AOrdering::Release);
}

/// `true` after [`register_main_thread_drainer`] — queued [`run_on_main`] work runs on the UI
/// thread instead of inline on arbitrary callers (e.g. timer threads).
pub fn main_queue_active() -> bool {
    MAIN_DRAINER_REGISTERED.load(AOrdering::Acquire)
}

/// Drain pending main-thread work. Should be called from the host's UI run loop
/// (e.g. once per frame). Safe to call when no work is pending.
pub fn drain_main_queue() {
    let mut items: Vec<WorkFn> = Vec::new();
    if let Ok(mut q) = scheduler().main_queue.lock() {
        std::mem::swap(&mut *q, &mut items);
    }
    for f in items {
        f();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex as StdMutex;

    static SCHED_TEST_LOCK: StdMutex<()> = StdMutex::new(());

    #[test]
    fn spawn_runs_on_worker_thread() {
        let _g = SCHED_TEST_LOCK.lock().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let _ = spawn(move || {
            c.fetch_add(1, AOrdering::Relaxed);
        });
        for _ in 0..50 {
            if counter.load(AOrdering::Relaxed) == 1 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(counter.load(AOrdering::Relaxed), 1);
    }

    #[test]
    fn spawn_delay_fires_once() {
        let _g = SCHED_TEST_LOCK.lock().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        spawn_delay(Duration::from_millis(20), move || {
            c.fetch_add(1, AOrdering::Relaxed);
        });
        for _ in 0..80 {
            if counter.load(AOrdering::Relaxed) == 1 {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(counter.load(AOrdering::Relaxed), 1);
    }

    #[test]
    fn spawn_interval_fires_multiple_times_until_cancel() {
        let _g = SCHED_TEST_LOCK.lock().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let id = spawn_interval(Duration::from_millis(20), move || {
            c.fetch_add(1, AOrdering::Relaxed);
        });
        thread::sleep(Duration::from_millis(100));
        cancel(id);
        let snap = counter.load(AOrdering::Relaxed);
        thread::sleep(Duration::from_millis(60));
        assert!(snap >= 2, "expected at least 2 fires, got {snap}");
        assert_eq!(counter.load(AOrdering::Relaxed), snap);
    }

    #[test]
    fn cancel_all_recurring_before_first_fire() {
        let _g = SCHED_TEST_LOCK.lock().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        let _ = spawn_interval(Duration::from_millis(200), move || {
            c.fetch_add(1, AOrdering::Relaxed);
        });
        thread::sleep(Duration::from_millis(20));
        cancel_all_recurring();
        thread::sleep(Duration::from_millis(250));
        assert_eq!(counter.load(AOrdering::Relaxed), 0);
    }

    #[test]
    fn run_on_main_inline_in_headless() {
        let _g = SCHED_TEST_LOCK.lock().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        run_on_main(move || {
            c.fetch_add(1, AOrdering::Relaxed);
        });
        assert_eq!(counter.load(AOrdering::Relaxed), 1);
    }

    #[test]
    fn peak_concurrent_jobs_bounded_by_worker_count() {
        let _g = SCHED_TEST_LOCK.lock().unwrap();
        for _ in 0..64 {
            spawn(|| {
                thread::sleep(Duration::from_millis(80));
            });
        }
        thread::sleep(Duration::from_millis(150));
        let peak = test_peak_active_jobs();
        assert!(
            peak <= MAX_DEFAULT_WORKERS,
            "peak concurrent jobs {peak} should not exceed max workers {MAX_DEFAULT_WORKERS}"
        );
    }

    #[test]
    fn spawn_blocking_completes() {
        let _g = SCHED_TEST_LOCK.lock().unwrap();
        let v = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&v);
        spawn_blocking(move || {
            c.fetch_add(1, AOrdering::Relaxed);
        });
        for _ in 0..50 {
            if v.load(AOrdering::Relaxed) == 1 {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("spawn_blocking did not complete");
    }

    #[test]
    fn cancel_skips_queued_spawn_before_run() {
        let _g = SCHED_TEST_LOCK.lock().unwrap();
        let w = worker_count().min(MAX_DEFAULT_WORKERS).max(1);
        let release = Arc::new(AtomicBool::new(false));
        let arrived = Arc::new(AtomicUsize::new(0));

        for _ in 0..w {
            let a = Arc::clone(&arrived);
            let r = Arc::clone(&release);
            spawn(move || {
                a.fetch_add(1, AOrdering::Relaxed);
                while !r.load(AOrdering::Acquire) {
                    thread::sleep(Duration::from_millis(1));
                }
            });
        }

        for _ in 0..200 {
            if arrived.load(AOrdering::Relaxed) >= w {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(
            arrived.load(AOrdering::Relaxed),
            w,
            "all workers should be busy"
        );

        let ran = Arc::new(AtomicBool::new(false));
        let r = Arc::clone(&ran);
        let id = spawn(move || {
            r.store(true, AOrdering::Release);
        });
        thread::sleep(Duration::from_millis(20));
        cancel(id);
        release.store(true, AOrdering::Release);
        thread::sleep(Duration::from_millis(100));
        assert!(
            !ran.load(AOrdering::Acquire),
            "cancelled queued job should not run"
        );
    }

    #[test]
    fn pool_stats_jobs_completed_increases() {
        let _g = SCHED_TEST_LOCK.lock().unwrap();
        let base = pool_stats().jobs_completed;
        for _ in 0..4 {
            spawn(|| {});
        }
        for _ in 0..100 {
            if pool_stats().jobs_completed >= base + 4 {
                return;
            }
            thread::sleep(Duration::from_millis(5));
        }
        panic!(
            "jobs_completed did not reach {} (last {:?})",
            base + 4,
            pool_stats()
        );
    }

    #[test]
    fn standalone_work_pool_shutdown_joins_threads() {
        let _g = SCHED_TEST_LOCK.lock().unwrap();
        let pool = super::WorkPool::new(2);
        thread::sleep(Duration::from_millis(20));
        pool.request_shutdown();
        pool.join_workers();
    }
}
