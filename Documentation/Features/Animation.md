# Animation Engine

Arcadia ships a single shared tween engine (`arcadia_core::modules::animation`). One 16 ms interval drives all running tweens — no per-animation timer overhead. The driver starts lazily on the first call and stops automatically when all tweens finish.

---

## Concepts

### Tween

A tween interpolates a value from 0.0 to 1.0 over a fixed duration. Every ~16 ms (one driver tick) the engine calls your `on_tick` callback with an eased `t ∈ [0.0, 1.0]`. You map that `t` to whatever property you are animating (color channel, opacity, position, height, etc.).

### Easing

Raw `t` is a linear ratio of elapsed / duration. An easing function bends that curve before you receive it.

| String key | Curve | Best for |
|---|---|---|
| `linear` | Constant rate | Progress bars, loaders |
| `ease_out_cubic` | Fast start, gentle finish | UI transitions, hover states |
| `ease_in_cubic` | Gentle start, fast finish | Dismiss / collapse |
| `ease_in_out_cubic` | Gentle both ends | Page transitions |
| `ease_out_elastic` | Overshoot + settle | Playful spring effect |

`EaseOutCubic` is the house default — used for all sidebar hover, active, caret, and settings expand animations.

### Math helpers

```rust
lerp_f32(a: f32, b: f32, t: f32) -> f32   // linear interpolate scalars
lerp_u8 (a: u8,  b: u8,  t: f32) -> u8    // same for byte channels
lerp_rgba(a: [u8;4], b: [u8;4], t: f32) -> [u8;4]  // RGBA array lerp
apply_easing(easing: Easing, t: f32) -> f32  // bend raw t through a curve
```

These are pure functions — no allocation, no engine interaction.

---

## Rust API

### Starting a tween

```rust
use arcadia_core::modules::animation::{tween, Easing, TweenId};

let id: TweenId = tween(
    300,                  // duration in milliseconds
    Easing::EaseOutCubic, // easing curve
    move |t| {           // called every ~16 ms with eased t ∈ [0.0, 1.0]
        let value = lerp_f32(start, end, t);
        // apply value to shared state
    },
);
```

`tween` returns a `TweenId` you can use to cancel early.

### With a completion callback

```rust
use arcadia_core::modules::animation::tween_with_completion;

tween_with_completion(
    300,
    Easing::EaseOutCubic,
    move |t| { /* tick */ },
    Some(Box::new(|| { /* fires once after final tick(1.0) */ })),
    None, // owner_extension — used by Python bindings only
);
```

### Cancelling

```rust
animation::cancel(id);         // cancel one tween
animation::cancel_all();       // cancel everything
animation::is_running(id);     // → bool
animation::running_count();    // → usize
```

---

## GUI integration pattern

The tween callback runs on the scheduler thread, not the render thread. It cannot hold `&mut ArcadiaRoot` or call `cx.notify()`. The Arcadia GUI uses a different pattern: **compute animations inline inside `render()` using `Instant` + `apply_easing` directly**, then drive re-renders with `window.request_animation_frame()`.

### Step 1 — Store animation state on the root

```rust
// In ArcadiaRoot (mod.rs)
pub struct CaretAnim {
    pub start: Instant,
    pub from: f32,
    pub to: f32,
}

pub my_alpha: f32,
pub my_anim: Option<CaretAnim>,
```

### Step 2 — Tick in `tick_caret_anims`

```rust
// In lifecycle.rs, inside tick_caret_anims()
const DURATION_S: f32 = 0.12; // 120 ms

if let Some(anim) = self.my_anim.clone() {
    let raw_t = (now - anim.start).as_secs_f32() / DURATION_S;
    let t = apply_easing(Easing::EaseOutCubic, raw_t.min(1.0));
    self.my_alpha = anim.from + (anim.to - anim.from) * t;
    if raw_t >= 1.0 {
        self.my_anim = None;
    } else {
        running = true; // request another frame
    }
}
```

### Step 3 — Start the animation from an event handler

```rust
pub fn start_my_anim(&mut self, show: bool) {
    use std::time::Instant;
    let from = self.my_alpha;
    let to   = if show { 1.0_f32 } else { 0.0_f32 };
    self.my_anim = Some(CaretAnim { start: Instant::now(), from, to });
}

// In a cx.listener callback:
cx.listener(move |this, hovered: &bool, _, cx| {
    this.start_my_anim(*hovered);
    cx.notify();
})
```

### Step 4 — Consume `my_alpha` in render

```rust
// tick_caret_anims is called at the top of render()
// returns true → window.request_animation_frame() is called → drives the next frame

.opacity(self.my_alpha)
// or use lerp_color / nav_item_bg to blend colors
```

`cx.notify()` from the event handler triggers the first render; `request_animation_frame()` drives all subsequent frames until `running` returns false.

---

## Nav item color formula

All sidebar nav items share one helper that encodes the four interactive states:

```rust
// nav_items.rs
pub(super) fn nav_item_bg(idle: Rgba, sel: Rgba, active_alpha: f32, hover_alpha: f32) -> Rgba {
    let idle_hov   = lerp_color(idle, sel, 0.4);                           // 40% toward active
    let active_hov = lerp_color(sel, Rgba { r:1., g:1., b:1., a:1. }, 0.05); // slightly lighter than active
    let base       = lerp_color(idle, sel, active_alpha);
    let dest       = lerp_color(idle_hov, active_hov, active_alpha);
    lerp_color(base, dest, hover_alpha)
}
```

| State | active_alpha | hover_alpha | Result |
|---|---|---|---|
| Idle | 0.0 | 0.0 | `idle` |
| Idle + hover | 0.0 | 1.0 | 40% toward active |
| Active | 1.0 | 0.0 | `sel` |
| Active + hover | 1.0 | 1.0 | `sel` lightened 5% |

Group tabs use a continuous `active_alpha` (0 → 1) animated via `tab_active_anims`. Page items use a boolean cast to 0.0 / 1.0. Both call `nav_item_bg` identically — consistent across the whole sidebar.

---

## Height-clip animation

To animate a container opening/closing (e.g. the settings hub pinned list):

```rust
// Compute target height from item count
let target_h = visible_count as f32 * 28.0; // 28 px per row
let animated_h = self.settings_expand_alpha * target_h;

div()
    .overflow_hidden()
    .h(px(animated_h))
    .flex()
    .flex_col()
    .children(items)
```

`overflow_hidden` clips content to the animated height. As `settings_expand_alpha` goes 0 → 1, the container slides open, revealing rows from top. `EaseOutCubic` makes it feel snappy at the start and settle gently.

---

## Python extension API

Python extensions drive tweens through two functions exposed on the `arcadia` module:

```python
import arcadia

# Start a tween — returns a numeric tween ID
tween_id = arcadia.animate(
    extension_id,   # str — your extension's ID (for scoped cancel on unload)
    duration_ms,    # int — total duration
    easing,         # str — one of the easing key strings above
    callback,       # callable(t: float) — receives eased t every ~16 ms
)

# Cancel early
arcadia.cancel_animation(tween_id)
```

**Threading note:** the callback runs on the Python timer lane. If you need to update shared overlay or sprite state, use thread-safe mechanisms. Do not hold the GIL across long operations inside the callback.

**Scoped cancellation:** when your extension is unloaded, all tweens registered under its `extension_id` are automatically cancelled — no manual cleanup required.

### Example — fade an overlay sprite in then out

```python
import arcadia

EXTENSION_ID = "my-extension"
SPRITE_ID    = "my-sprite"

def on_load():
    tween_id = arcadia.animate(
        EXTENSION_ID, 400, "ease_out_cubic",
        lambda t: arcadia.set_sprite_opacity(SPRITE_ID, t)
    )

def on_unload():
    # All tweens for EXTENSION_ID are cancelled automatically.
    pass
```

---

## CLI inspection

The `animation` module exposes commands for runtime debugging:

```
animation.list        → JSON array of running tweens (id, elapsed_ms, duration_ms, easing, owner)
animation.cancel <id> → cancel a specific tween by numeric id
animation.cancel_all  → cancel all running tweens
```

Example output of `animation.list`:

```json
[
  { "id": 7, "elapsed_ms": 43, "duration_ms": 300, "easing": "ease_out_cubic", "owner": "native" },
  { "id": 9, "elapsed_ms": 11, "duration_ms": 400, "easing": "ease_out_elastic", "owner": "my-extension" }
]
```

---

## Rules and constraints

- **Never store credentials or secrets in tween closures** — closures are heap-allocated and may appear in debug output.
- **Tween callbacks are `Send + Sync + 'static`** — captures must be `Arc<T>`, not `Rc<T>` or `&mut`.
- **GUI state mutation from a tween callback is not safe** — use the `Instant`-based inline pattern (step 1–4 above) for any animation that touches `ArcadiaRoot`.
- **`tween` is fire-and-forget for background work** — suitable for non-GUI animations (overlay sprites, tray icon pulses, audio visualizer values).
- **Duration 0** is valid: the engine fires `on_tick(1.0)` immediately on the next driver tick, then removes the tween. Useful for triggering a completion callback with a guaranteed single call.
