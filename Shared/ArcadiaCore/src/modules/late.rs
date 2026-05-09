use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};

use crate::config::late::LateConfig;
use crate::config::ConfigFile;
use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "late";

// ── Domain types ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LateReaction {
    pub emoji: String,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateMessage {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub body: String,
    pub timestamp: String,
    pub reactions: Vec<LateReaction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateUser {
    pub user_id: u32,
    pub username: String,
    pub room_id: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LateNowPlaying {
    pub track: String,
    pub artist: String,
    pub album: String,
    pub progress_sec: u32,
    pub duration_sec: u32,
    pub volume_pct: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LateVotes {
    pub lofi: u32,
    pub ambient: u32,
    pub classic: u32,
    pub jazz: u32,
    pub next_vote_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateActivityEvent {
    pub kind: String,
    pub username: String,
    pub room_id: u32,
    pub timestamp: String,
}

// ── Shared state ──────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct LateState {
    pub connected: bool,
    pub active_room: u32,
    /// Bounded to the last 200 messages (across all subscribed rooms).
    pub messages: VecDeque<LateMessage>,
    pub online_users: Vec<LateUser>,
    pub now_playing: LateNowPlaying,
    pub votes: LateVotes,
    pub visualizer_frame: String,
    pub bonsai_art: Vec<String>,
    pub activity_feed: VecDeque<LateActivityEvent>,
    pub connection_error: Option<String>,
    /// Incremented on every mutation so the GUI poll task knows when to repaint.
    pub revision: u64,
}

const MAX_MESSAGES: usize = 200;
const MAX_ACTIVITY: usize = 50;

impl LateState {
    fn push_message(&mut self, msg: LateMessage) {
        if self.messages.len() >= MAX_MESSAGES {
            self.messages.pop_front();
        }
        self.messages.push_back(msg);
        self.revision += 1;
    }

    fn push_activity(&mut self, event: LateActivityEvent) {
        if self.activity_feed.len() >= MAX_ACTIVITY {
            self.activity_feed.pop_front();
        }
        self.activity_feed.push_back(event);
        self.revision += 1;
    }
}

static STATE: OnceLock<Arc<Mutex<LateState>>> = OnceLock::new();

pub fn state() -> Arc<Mutex<LateState>> {
    STATE
        .get_or_init(|| Arc::new(Mutex::new(LateState::default())))
        .clone()
}

// ── WebSocket background thread ───────────────────────────────────────────────

static WS_RUNNING: AtomicBool = AtomicBool::new(false);
static WS_SENDER: OnceLock<Mutex<Option<mpsc::Sender<String>>>> = OnceLock::new();

fn ws_sender_lock() -> &'static Mutex<Option<mpsc::Sender<String>>> {
    WS_SENDER.get_or_init(|| Mutex::new(None))
}

pub fn start_ws_thread(server_url: String, ticket: String) {
    if WS_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::Builder::new()
        .name("late-ws".to_string())
        .spawn(move || ws_loop(server_url, ticket))
        .ok();
}

pub fn stop_ws_thread() {
    WS_RUNNING.store(false, Ordering::SeqCst);
    if let Ok(mut guard) = ws_sender_lock().lock() {
        *guard = None;
    }
}

pub fn send_ws(msg: String) {
    if let Ok(guard) = ws_sender_lock().lock() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(msg);
        }
    }
}

fn ws_loop(server_url: String, ticket: String) {
    use tungstenite::{Message, connect};

    let (ws_scheme, host) = if let Some(host) = server_url.strip_prefix("https://") {
        ("wss", host)
    } else if let Some(host) = server_url.strip_prefix("http://") {
        ("ws", host)
    } else {
        ("wss", server_url.as_str())
    };
    let ws_url = format!("{ws_scheme}://{host}/api/ws/native?ticket={ticket}");
    eprintln!("[late] ws connecting");

    let (mut socket, _) = match connect(ws_url.as_str()) {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!("[late] ws connect failed: {err}");
            let arc = state();
            let mut st = arc.lock().unwrap_or_else(|e| e.into_inner());
            st.connection_error = Some(format!("WS connect failed: {err}"));
            st.connected = false;
            st.revision += 1;
            WS_RUNNING.store(false, Ordering::SeqCst);
            return;
        }
    };
    eprintln!("[late] ws connected");

    {
        let arc = state();
        let mut st = arc.lock().unwrap_or_else(|e| e.into_inner());
        st.connected = true;
        st.connection_error = None;
        st.revision += 1;
    }

    let (tx, rx) = mpsc::channel::<String>();
    {
        let mut guard = ws_sender_lock().lock().unwrap_or_else(|e| e.into_inner());
        *guard = Some(tx);
    }

    // Sender sub-thread: drains mpsc channel → WS write.
    // We can't share the socket across threads, so we use a second connection-side approach:
    // instead, handle sends inline in the read loop by making the socket non-blocking on reads
    // isn't possible with tungstenite blocking mode. Use a simple approach: poll the channel
    // before each read with try_recv.

    loop {
        if !WS_RUNNING.load(Ordering::Relaxed) {
            break;
        }

        // Drain outbound queue first.
        while let Ok(outbound) = rx.try_recv() {
            if let Err(err) = socket.send(Message::Text(outbound)) {
                eprintln!("[late] ws send failed: {err}");
                break;
            }
        }

        // Set a short read timeout so we can interleave outbound sends.
        match socket.read() {
            Ok(Message::Text(text)) => handle_ws_message(&text),
            Ok(Message::Ping(data)) => {
                let _ = socket.send(Message::Pong(data));
            }
            Ok(Message::Close(_)) => {
                eprintln!("[late] ws closed by remote");
                break;
            }
            Err(err) => {
                eprintln!("[late] ws read error: {err}");
                break;
            }
            Ok(_) => {}
        }
    }

    {
        let arc = state();
        let mut st = arc.lock().unwrap_or_else(|e| e.into_inner());
        st.connected = false;
        st.revision += 1;
    }
    {
        let mut guard = ws_sender_lock().lock().unwrap_or_else(|e| e.into_inner());
        *guard = None;
    }
    WS_RUNNING.store(false, Ordering::SeqCst);
    eprintln!("[late] ws disconnected");
}

fn handle_ws_message(text: &str) {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(text) else {
        return;
    };
    let msg_type = val["type"].as_str().unwrap_or("");
    let arc = state();
    let mut st = arc.lock().unwrap_or_else(|e| e.into_inner());

    match msg_type {
        "init" => {
            if let Some(users) = val["online_users"].as_array() {
                st.online_users = users
                    .iter()
                    .filter_map(|u| serde_json::from_value(u.clone()).ok())
                    .collect();
            }
            if let Ok(np) = serde_json::from_value::<LateNowPlaying>(val["now_playing"].clone()) {
                st.now_playing = np;
            }
            if let Ok(votes) = serde_json::from_value::<LateVotes>(val["votes"].clone()) {
                st.votes = votes;
            }
            if let Some(messages) = val["messages"].as_array() {
                st.messages.clear();
                for raw in messages {
                    if let Some(msg) = parse_ws_message(raw) {
                        st.push_message(msg);
                    }
                }
            }
            st.revision += 1;
        }
        "message" => {
            if let Some(msg) = parse_ws_message(&val["msg"]) {
                st.push_message(msg);
            }
        }
        "reaction_update" => {
            let msg_id = val["msg_id"]
                .as_str()
                .map(str::to_string)
                .or_else(|| val["msg_id"].as_u64().map(|n| n.to_string()))
                .unwrap_or_default();
            if let Some(reactions) =
                serde_json::from_value::<Vec<LateReaction>>(val["reactions"].clone()).ok()
            {
                if let Some(m) = st.messages.iter_mut().find(|m| m.id == msg_id) {
                    m.reactions = reactions;
                    st.revision += 1;
                }
            }
        }
        "presence" => {
            let event = LateActivityEvent {
                kind: val["event"].as_str().unwrap_or("join").to_string(),
                username: val["username"].as_str().unwrap_or("").to_string(),
                room_id: val["room_id"].as_u64().unwrap_or(0) as u32,
                timestamp: val["timestamp"].as_str().unwrap_or("").to_string(),
            };
            let username = event.username.clone();
            let user_id = val["user_id"].as_u64().unwrap_or(0) as u32;
            let room_id = event.room_id;
            st.push_activity(event);
            match val["event"].as_str().unwrap_or("") {
                "join" => {
                    if !st.online_users.iter().any(|u| u.user_id == user_id) {
                        st.online_users.push(LateUser {
                            user_id,
                            username,
                            room_id: Some(room_id),
                        });
                    }
                }
                "leave" => {
                    st.online_users.retain(|u| u.user_id != user_id);
                }
                _ => {}
            }
            st.revision += 1;
        }
        "now_playing" => {
            if let Ok(np) = serde_json::from_value::<LateNowPlaying>(val.clone()) {
                st.now_playing = np;
                st.revision += 1;
            }
        }
        "votes" => {
            if let Ok(votes) = serde_json::from_value::<LateVotes>(val.clone()) {
                st.votes = votes;
                st.revision += 1;
            }
        }
        "visualizer" => {
            if let Some(frame) = val["frame"].as_str() {
                st.visualizer_frame = frame.to_string();
                st.revision += 1;
            }
        }
        "bonsai" => {
            if let Some(art) = val["art"].as_array() {
                st.bonsai_art = art
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                st.revision += 1;
            }
        }
        "ping" => {
            send_ws(r#"{"type":"pong"}"#.to_string());
        }
        _ => {}
    }
}

fn parse_ws_message(raw: &serde_json::Value) -> Option<LateMessage> {
    Some(LateMessage {
        id: raw["id"]
            .as_str()
            .map(str::to_string)
            .or_else(|| raw["id"].as_u64().map(|n| n.to_string()))?,
        user_id: raw["user_id"]
            .as_str()
            .map(str::to_string)
            .or_else(|| raw["user_id"].as_u64().map(|n| n.to_string()))
            .unwrap_or_default(),
        username: raw["username"].as_str().unwrap_or("").to_string(),
        body: raw["body"].as_str().unwrap_or("").to_string(),
        timestamp: raw["timestamp"].as_str().unwrap_or("").to_string(),
        reactions: serde_json::from_value::<Vec<LateReaction>>(raw["reactions"].clone())
            .unwrap_or_default(),
    })
}

// ── HTTP helpers ──────────────────────────────────────────────────────────────

pub fn http_get_challenge(server_url: &str) -> Result<String, String> {
    let url = format!("{server_url}/api/native/challenge");
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())?;
    resp["nonce"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "no nonce in response".to_string())
}

pub fn http_post_token(
    server_url: &str,
    fingerprint: &str,
    public_key: &str,
    nonce: &str,
    signature_pem: &str,
) -> Result<String, String> {
    let url = format!("{server_url}/api/native/token");
    let body = serde_json::json!({
        "public_key_fingerprint": fingerprint,
        "public_key": public_key,
        "nonce": nonce,
        "signature_pem": signature_pem,
    });
    let resp = match ureq::post(&url).send_json(body) {
        Ok(resp) => resp,
        Err(ureq::Error::Status(code, response)) => {
            let body = response
                .into_string()
                .unwrap_or_else(|_| "<failed to read error body>".to_string());
            return Err(format!("token endpoint returned {code}: {body}"));
        }
        Err(err) => return Err(err.to_string()),
    }
    .into_json::<serde_json::Value>()
    .map_err(|e| e.to_string())?;
    resp["token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "no token in response".to_string())
}

pub fn http_get_now_playing(server_url: &str, token: &str) -> Result<LateNowPlaying, String> {
    let url = format!("{server_url}/api/native/now-playing");
    ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<LateNowPlaying>()
        .map_err(|e| e.to_string())
}

pub fn http_post_vote(server_url: &str, token: &str, genre: &str) -> Result<LateVotes, String> {
    let url = format!("{server_url}/api/native/vote");
    ureq::post(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .send_json(serde_json::json!({"genre": genre}))
        .map_err(|e| e.to_string())?
        .into_json::<LateVotes>()
        .map_err(|e| e.to_string())
}

pub fn http_get_history(
    server_url: &str,
    token: &str,
    room_id: u32,
    limit: usize,
) -> Result<Vec<LateMessage>, String> {
    let url = format!("{server_url}/api/native/rooms/{room_id}/history?limit={limit}");
    ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<Vec<LateMessage>>()
        .map_err(|e| e.to_string())
}

pub fn http_post_send(
    server_url: &str,
    token: &str,
    room_id: u32,
    body: &str,
) -> Result<(), String> {
    let url = format!("{server_url}/api/native/rooms/{room_id}/messages");
    ureq::post(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .send_json(serde_json::json!({"body": body}))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn http_post_water_bonsai(server_url: &str, token: &str) -> Result<Vec<String>, String> {
    let url = format!("{server_url}/api/native/bonsai/water");
    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())?;
    resp["art"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .ok_or_else(|| "no art in response".to_string())
}

pub fn http_get_bonsai(server_url: &str, token: &str) -> Result<Vec<String>, String> {
    let url = format!("{server_url}/api/native/bonsai");
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())?;
    resp["art"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .ok_or_else(|| "no art in response".to_string())
}

pub fn http_get_ws_ticket(server_url: &str, token: &str) -> Result<String, String> {
    let url = format!("{server_url}/api/native/ws-ticket");
    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())?;
    resp["ticket"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "no ticket in response".to_string())
}

pub fn http_delete_token(server_url: &str, token: &str) -> Result<(), String> {
    let url = format!("{server_url}/api/native/logout");
    match ureq::delete(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
    {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(code, response)) => {
            let body = response.into_string().unwrap_or_default();
            Err(format!("logout returned {code}: {body}"))
        }
        Err(e) => Err(e.to_string()),
    }
}

// ── Experimental state (late.experimental page) ───────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LateProfile {
    pub username: String,
    pub bio: String,
    pub notify_format: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateNotification {
    pub id: String,
    pub kind: String,
    pub body: String,
    pub created: String,
    pub read: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateArticle {
    pub id: String,
    pub url: String,
    pub title: String,
    pub summary: String,
    pub created: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateWorkProfile {
    pub id: String,
    pub slug: String,
    pub headline: String,
    pub status: String,
    pub work_type: String,
    pub location: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateRssFeed {
    pub id: String,
    pub url: String,
    pub title: String,
    pub active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateRssEntry {
    pub id: String,
    pub feed_title: String,
    pub title: String,
    pub url: String,
    pub published_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateShowcaseItem {
    pub id: String,
    pub title: String,
    pub url: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateLeaderEntry {
    pub username: String,
    pub game: String,
    pub score: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LateChip {
    pub id: String,
    pub label: String,
    pub color: String,
    pub count: i64,
}

#[derive(Default)]
pub struct LateExperimentalState {
    pub profile: Option<LateProfile>,
    pub notifications: Vec<LateNotification>,
    pub unread_notifications: i64,
    pub articles: Vec<LateArticle>,
    pub work_profiles: Vec<LateWorkProfile>,
    pub rss_feeds: Vec<LateRssFeed>,
    pub rss_unread: i64,
    pub rss_entries: Vec<LateRssEntry>,
    pub showcase: Vec<LateShowcaseItem>,
    pub leaderboard: Vec<LateLeaderEntry>,
    pub chips: Vec<LateChip>,
    pub artboard_size: Option<(u32, u32)>,
    pub last_error: Option<String>,
    pub loading: bool,
    pub revision: u64,
}

static EXPERIMENTAL_STATE: OnceLock<Arc<Mutex<LateExperimentalState>>> = OnceLock::new();

pub fn experimental_state() -> Arc<Mutex<LateExperimentalState>> {
    EXPERIMENTAL_STATE
        .get_or_init(|| Arc::new(Mutex::new(LateExperimentalState::default())))
        .clone()
}

pub fn experimental_refresh(server_url: String, token: String) {
    {
        let arc = experimental_state();
        let mut st = arc.lock().unwrap_or_else(|e| e.into_inner());
        st.loading = true;
        st.last_error = None;
        st.revision += 1;
    }
    std::thread::Builder::new()
        .name("late-exp-refresh".to_string())
        .spawn(move || {
            let mut errors: Vec<String> = Vec::new();

            macro_rules! fetch {
                ($result:expr, $apply:expr) => {
                    match $result {
                        Ok(val) => {
                            let arc = experimental_state();
                            let mut st = arc.lock().unwrap_or_else(|e| e.into_inner());
                            $apply(&mut *st, val);
                            st.revision += 1;
                        }
                        Err(e) => errors.push(e),
                    }
                };
            }

            fetch!(
                http_get_profile(&server_url, &token),
                |st: &mut LateExperimentalState, v: LateProfile| { st.profile = Some(v); }
            );
            fetch!(
                http_get_notifications(&server_url, &token),
                |st: &mut LateExperimentalState, v: Vec<LateNotification>| { st.notifications = v; }
            );
            fetch!(
                http_get_notifications_unread(&server_url, &token),
                |st: &mut LateExperimentalState, v: i64| { st.unread_notifications = v; }
            );
            fetch!(
                http_get_articles(&server_url, &token),
                |st: &mut LateExperimentalState, v: Vec<LateArticle>| { st.articles = v; }
            );
            fetch!(
                http_get_work_profiles(&server_url, &token),
                |st: &mut LateExperimentalState, v: Vec<LateWorkProfile>| { st.work_profiles = v; }
            );
            fetch!(
                http_get_rss_feeds(&server_url, &token),
                |st: &mut LateExperimentalState, v: Vec<LateRssFeed>| { st.rss_feeds = v; }
            );
            fetch!(
                http_get_rss_unread(&server_url, &token),
                |st: &mut LateExperimentalState, v: i64| { st.rss_unread = v; }
            );
            fetch!(
                http_get_rss_entries(&server_url, &token),
                |st: &mut LateExperimentalState, v: Vec<LateRssEntry>| { st.rss_entries = v; }
            );
            fetch!(
                http_get_showcase(&server_url, &token),
                |st: &mut LateExperimentalState, v: Vec<LateShowcaseItem>| { st.showcase = v; }
            );
            fetch!(
                http_get_leaderboard(&server_url, &token),
                |st: &mut LateExperimentalState, v: Vec<LateLeaderEntry>| { st.leaderboard = v; }
            );
            fetch!(
                http_get_chips(&server_url, &token),
                |st: &mut LateExperimentalState, v: Vec<LateChip>| { st.chips = v; }
            );
            fetch!(
                http_get_artboard_size(&server_url, &token),
                |st: &mut LateExperimentalState, v: Option<(u32, u32)>| { st.artboard_size = v; }
            );

            let arc = experimental_state();
            let mut st = arc.lock().unwrap_or_else(|e| e.into_inner());
            st.loading = false;
            if !errors.is_empty() {
                st.last_error = Some(errors.join("; "));
            }
            st.revision += 1;
        })
        .ok();
}

// ── Experimental HTTP helpers ─────────────────────────────────────────────────

fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

pub fn http_get_profile(server_url: &str, token: &str) -> Result<LateProfile, String> {
    ureq::get(&format!("{server_url}/api/native/profile"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| LateProfile {
            username: v["username"].as_str().unwrap_or("").to_string(),
            bio: v["bio"].as_str().unwrap_or("").to_string(),
            notify_format: v["notify_format"].as_str().unwrap_or("").to_string(),
        })
}

pub fn http_get_notifications(server_url: &str, token: &str) -> Result<Vec<LateNotification>, String> {
    ureq::get(&format!("{server_url}/api/native/notifications"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|n| serde_json::from_value(n.clone()).ok())
                        .collect()
                })
                .unwrap_or_default()
        })
}

pub fn http_get_notifications_unread(server_url: &str, token: &str) -> Result<i64, String> {
    ureq::get(&format!("{server_url}/api/native/notifications/unread"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| v["unread"].as_i64().unwrap_or(0))
}

pub fn http_get_articles(server_url: &str, token: &str) -> Result<Vec<LateArticle>, String> {
    ureq::get(&format!("{server_url}/api/native/articles?limit=10"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|a| LateArticle {
                            id: a["id"].as_str().unwrap_or("").to_string(),
                            url: a["url"].as_str().unwrap_or("").to_string(),
                            title: a["title"].as_str().unwrap_or("").to_string(),
                            summary: a["summary"].as_str().unwrap_or("").to_string(),
                            created: a["created"].as_str().unwrap_or("").to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default()
        })
}

pub fn http_get_work_profiles(server_url: &str, token: &str) -> Result<Vec<LateWorkProfile>, String> {
    ureq::get(&format!("{server_url}/api/native/work-profiles?limit=20"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|p| LateWorkProfile {
                            id: p["id"].as_str().unwrap_or("").to_string(),
                            slug: p["slug"].as_str().unwrap_or("").to_string(),
                            headline: p["headline"].as_str().unwrap_or("").to_string(),
                            status: p["status"].as_str().unwrap_or("").to_string(),
                            work_type: p["work_type"].as_str().unwrap_or("").to_string(),
                            location: p["location"].as_str().unwrap_or("").to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default()
        })
}

pub fn http_get_rss_feeds(server_url: &str, token: &str) -> Result<Vec<LateRssFeed>, String> {
    ureq::get(&format!("{server_url}/api/native/rss/feeds"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|f| LateRssFeed {
                            id: f["id"].as_str().unwrap_or("").to_string(),
                            url: f["url"].as_str().unwrap_or("").to_string(),
                            title: f["title"].as_str().unwrap_or("").to_string(),
                            active: f["active"].as_bool().unwrap_or(false),
                        })
                        .collect()
                })
                .unwrap_or_default()
        })
}

pub fn http_get_rss_unread(server_url: &str, token: &str) -> Result<i64, String> {
    ureq::get(&format!("{server_url}/api/native/rss/unread"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| v["unread"].as_i64().unwrap_or(0))
}

pub fn http_get_rss_entries(server_url: &str, token: &str) -> Result<Vec<LateRssEntry>, String> {
    ureq::get(&format!("{server_url}/api/native/rss/entries?limit=20"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|e| LateRssEntry {
                            id: e["id"].as_str().unwrap_or("").to_string(),
                            feed_title: e["feed_title"].as_str().unwrap_or("").to_string(),
                            title: e["title"].as_str().unwrap_or("").to_string(),
                            url: e["url"].as_str().unwrap_or("").to_string(),
                            published_at: e["published_at"].as_str().map(|s| s.to_string()),
                        })
                        .collect()
                })
                .unwrap_or_default()
        })
}

pub fn http_get_showcase(server_url: &str, token: &str) -> Result<Vec<LateShowcaseItem>, String> {
    ureq::get(&format!("{server_url}/api/native/showcase?limit=20"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|s| LateShowcaseItem {
                            id: s["id"].as_str().unwrap_or("").to_string(),
                            title: s["title"].as_str().unwrap_or("").to_string(),
                            url: s["url"].as_str().unwrap_or("").to_string(),
                            description: s["description"].as_str().unwrap_or("").to_string(),
                            tags: s["tags"]
                                .as_array()
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default(),
                        })
                        .collect()
                })
                .unwrap_or_default()
        })
}

pub fn http_get_leaderboard(server_url: &str, token: &str) -> Result<Vec<LateLeaderEntry>, String> {
    ureq::get(&format!("{server_url}/api/native/games/leaderboard"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|e| LateLeaderEntry {
                            username: e["username"].as_str().unwrap_or("").to_string(),
                            game: e["game"].as_str().unwrap_or("").to_string(),
                            score: e["score"].as_i64().unwrap_or(0),
                        })
                        .collect()
                })
                .unwrap_or_default()
        })
}

pub fn http_get_chips(server_url: &str, token: &str) -> Result<Vec<LateChip>, String> {
    ureq::get(&format!("{server_url}/api/native/chips"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())
        .map(|v| {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|c| LateChip {
                            id: c["id"].as_str().unwrap_or("").to_string(),
                            label: c["label"].as_str().unwrap_or("").to_string(),
                            color: c["color"].as_str().unwrap_or("").to_string(),
                            count: c["count"].as_i64().unwrap_or(0),
                        })
                        .collect()
                })
                .unwrap_or_default()
        })
}

pub fn http_get_artboard_size(server_url: &str, token: &str) -> Result<Option<(u32, u32)>, String> {
    let val = ureq::get(&format!("{server_url}/api/native/artboard"))
        .set("Authorization", &bearer(token))
        .call()
        .map_err(|e| e.to_string())?
        .into_json::<serde_json::Value>()
        .map_err(|e| e.to_string())?;
    let canvas = &val["canvas"];
    let w = canvas["width"].as_u64().or_else(|| canvas["cols"].as_u64());
    let h = canvas["height"].as_u64().or_else(|| canvas["rows"].as_u64());
    Ok(w.zip(h).map(|(w, h)| (w as u32, h as u32)))
}

fn expand_tilde_path(path: &str) -> String {
    if path == "~" {
        return std::env::var("HOME").unwrap_or_else(|_| path.to_string());
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    path.to_string()
}

// ── Module commands ───────────────────────────────────────────────────────────

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "connect",
            description: "connect to late.sh WebSocket (reads server_url and auth_token from late.toml)",
            run: cmd_connect,
        },
        ModuleCommand {
            name: "disconnect",
            description: "disconnect from late.sh WebSocket",
            run: cmd_disconnect,
        },
        ModuleCommand {
            name: "status",
            description: "connection status and current now-playing as JSON",
            run: cmd_status,
        },
        ModuleCommand {
            name: "send",
            description: "late.send <room_id> <message...> — send a chat message",
            run: cmd_send,
        },
        ModuleCommand {
            name: "vote",
            description: "late.vote lofi|ambient|classic — vote for next music genre",
            run: cmd_vote,
        },
        ModuleCommand {
            name: "water",
            description: "water bonsai and refresh bonsai art",
            run: cmd_water,
        },
        ModuleCommand {
            name: "login",
            description: "late.login <ssh_key_path> — exchange SSH key for API token (desktop only)",
            run: cmd_login,
        },
        ModuleCommand {
            name: "logout",
            description: "revoke the stored API token and clear it from late.toml",
            run: cmd_logout,
        },
    ]
}

fn cmd_connect(args: &[&str], _ctx: &ExecutionContext) -> String {
    let cfg = match LateConfig::load_or_create() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[late] failed to load late.toml: {e}");
            return format!("error: failed to load late.toml: {e}");
        }
    };
    if cfg.auth_token.is_empty() {
        eprintln!("[late] connect blocked: missing auth_token in late.toml");
        return "error: no auth_token in late.toml — run late.login first".to_string();
    }
    let server_url = if let Some(url) = args.first() {
        url.to_string()
    } else {
        cfg.server_url.clone()
    };
    eprintln!("[late] connect requested: server_url={server_url}");

    match http_get_bonsai(&cfg.server_url, &cfg.auth_token) {
        Ok(art) => {
            let arc = state();
            let mut st = arc.lock().unwrap_or_else(|e| e.into_inner());
            st.bonsai_art = art;
            st.revision += 1;
        }
        Err(err) => eprintln!("[late] bonsai prefetch failed: {err}"),
    }

    let ticket = match http_get_ws_ticket(&cfg.server_url, &cfg.auth_token) {
        Ok(t) => t,
        Err(e) => return format!("error: failed to get WS ticket: {e}"),
    };

    start_ws_thread(server_url, ticket);
    "connecting to late.sh…".to_string()
}

fn cmd_disconnect(_args: &[&str], _ctx: &ExecutionContext) -> String {
    stop_ws_thread();
    "disconnected".to_string()
}

fn cmd_status(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let st = state();
    let guard = st.lock().unwrap_or_else(|e| e.into_inner());
    serde_json::json!({
        "connected": guard.connected,
        "revision": guard.revision,
        "active_room": guard.active_room,
        "now_playing": guard.now_playing,
        "votes": guard.votes,
        "online_users": guard.online_users.len(),
        "messages": guard.messages.len(),
        "error": guard.connection_error,
    })
    .to_string()
}

fn cmd_send(args: &[&str], _ctx: &ExecutionContext) -> String {
    if args.len() < 2 {
        return "usage: late.send <room_id> <message...>".to_string();
    }
    let room_id: u32 = match args[0].parse() {
        Ok(n) => n,
        Err(_) => return "error: room_id must be a number".to_string(),
    };
    let body = args[1..].join(" ");

    // Try WS first (fast path), fall back to HTTP.
    if WS_RUNNING.load(Ordering::Relaxed) {
        let msg = serde_json::json!({"type":"send","room_id":room_id,"body":body}).to_string();
        send_ws(msg);
        return "sent".to_string();
    }

    let cfg = match LateConfig::load_or_create() {
        Ok(c) => c,
        Err(e) => return format!("error: {e}"),
    };
    match http_post_send(&cfg.server_url, &cfg.auth_token, room_id, &body) {
        Ok(()) => "sent".to_string(),
        Err(e) => format!("error: {e}"),
    }
}

fn cmd_vote(args: &[&str], _ctx: &ExecutionContext) -> String {
    let genre = match args.first() {
        Some(g) => *g,
        None => return "usage: late.vote lofi|ambient|classic|jazz".to_string(),
    };
    if !["lofi", "ambient", "classic", "jazz"].contains(&genre) {
        return format!("error: unknown genre '{genre}' — use lofi, ambient, classic, or jazz");
    }

    if WS_RUNNING.load(Ordering::Relaxed) {
        let msg = serde_json::json!({"type":"vote","genre":genre}).to_string();
        send_ws(msg);
        return format!("voted {genre}");
    }

    let cfg = match LateConfig::load_or_create() {
        Ok(c) => c,
        Err(e) => return format!("error: {e}"),
    };
    match http_post_vote(&cfg.server_url, &cfg.auth_token, genre) {
        Ok(votes) => format!(
            "voted {genre} — lofi: {}, ambient: {}, classic: {}, jazz: {}",
            votes.lofi, votes.ambient, votes.classic, votes.jazz
        ),
        Err(e) => format!("error: {e}"),
    }
}

fn cmd_water(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let cfg = match LateConfig::load_or_create() {
        Ok(c) => c,
        Err(e) => return format!("error: {e}"),
    };
    match http_post_water_bonsai(&cfg.server_url, &cfg.auth_token) {
        Ok(art) => {
            let arc = state();
            let mut st = arc.lock().unwrap_or_else(|e| e.into_inner());
            st.bonsai_art = art;
            st.revision += 1;
            "bonsai watered".to_string()
        }
        Err(e) => format!("error: {e}"),
    }
}

fn cmd_logout(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let mut cfg = match LateConfig::load_or_create() {
        Ok(c) => c,
        Err(e) => return format!("error: failed to load late.toml: {e}"),
    };
    if cfg.auth_token.is_empty() {
        return "no token stored — already logged out".to_string();
    }
    // Best-effort revocation; don't fail logout if the server is unreachable.
    if let Err(e) = http_delete_token(&cfg.server_url, &cfg.auth_token) {
        eprintln!("[late] logout: server revocation failed (continuing): {e}");
    }
    stop_ws_thread();
    cfg.auth_token = String::new();
    match cfg.save() {
        Ok(()) => "logged out — token revoked and cleared from late.toml".to_string(),
        Err(e) => format!("token revoked on server but failed to clear late.toml: {e}"),
    }
}

fn cmd_login(args: &[&str], _ctx: &ExecutionContext) -> String {
    let key_path_input = match args.first() {
        Some(p) => *p,
        None => return "usage: late.login <ssh_key_path>".to_string(),
    };
    let key_path = expand_tilde_path(key_path_input);
    eprintln!("[late] login key path: input='{key_path_input}' resolved='{key_path}'");

    let cfg = match LateConfig::load_or_create() {
        Ok(c) => c,
        Err(e) => return format!("error: failed to load late.toml: {e}"),
    };

    // Step 1: get nonce.
    let nonce = match http_get_challenge(&cfg.server_url) {
        Ok(n) => n,
        Err(e) => return format!("error: challenge request failed: {e}"),
    };

    // Step 2: sign nonce bytes with ssh-keygen.
    let tmp_dir = std::env::temp_dir();
    let nonce_file = tmp_dir.join("late_nonce.bin");
    // Write raw nonce hex as bytes (server expects the hex string signed, not decoded bytes).
    if let Err(e) = std::fs::write(&nonce_file, nonce.as_bytes()) {
        return format!("error: failed to write nonce temp file: {e}");
    }

    let sign_output = std::process::Command::new("ssh-keygen")
        .args([
            "-Y", "sign",
            "-f", key_path.as_str(),
            "-n", "late.sh",
            nonce_file.to_str().unwrap_or(""),
        ])
        .output();

    let _ = std::fs::remove_file(&nonce_file);

    let sig_file = tmp_dir.join("late_nonce.bin.sig");
    let signature_pem = match sign_output {
        Ok(out) if out.status.success() => {
            match std::fs::read_to_string(&sig_file) {
                Ok(s) => { let _ = std::fs::remove_file(&sig_file); s }
                Err(e) => return format!("error: could not read signature file: {e}"),
            }
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return format!("error: ssh-keygen sign failed: {stderr}");
        }
        Err(e) => return format!("error: ssh-keygen not found or failed: {e}"),
    };

    // Step 3: read the public key and extract the fingerprint.
    let pub_key_path = format!("{key_path}.pub");
    let public_key = match std::fs::read_to_string(&pub_key_path) {
        Ok(s) => s.trim().to_string(),
        Err(e) => return format!("error: could not read public key {pub_key_path}: {e}"),
    };
    let fp_output = std::process::Command::new("ssh-keygen")
        .args(["-lf", key_path.as_str()])
        .output();
    let fingerprint = match fp_output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Format: "256 SHA256:xxxxx comment (ED25519)" — extract SHA256:xxx part.
            stdout
                .split_whitespace()
                .find(|s| s.starts_with("SHA256:"))
                .unwrap_or("")
                .to_string()
        }
        _ => return "error: ssh-keygen -lf failed — check key path".to_string(),
    };

    if fingerprint.is_empty() {
        return "error: could not extract fingerprint from ssh-keygen output".to_string();
    }
    eprintln!("[late] login fingerprint: {fingerprint}");

    // Step 4: exchange for token.
    let token = match http_post_token(&cfg.server_url, &fingerprint, &public_key, &nonce, &signature_pem) {
        Ok(t) => t,
        Err(e) => return format!("error: token exchange failed: {e}"),
    };

    // Step 5: save to config.
    let mut new_cfg = cfg;
    new_cfg.auth_token = token;
    match new_cfg.save() {
        Ok(()) => format!("logged in — token saved to late.toml (fingerprint: {fingerprint})"),
        Err(e) => format!("error: token received but failed to save late.toml: {e}"),
    }
}

