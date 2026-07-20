# Alone Engine

A lightweight 2D game engine written in Rust, built for pixel-art games with a small, fixed-resolution software-composited renderer. Component-based architecture powered by derive macros, kinematic physics, tilemap/LDtk support, and built-in networking.

> Alone Engine is a from-scratch redesign of milim-2d, rebuilt around a pure-Rust rendering stack (`winit` + `pixels`) and a trait-based engine context.

## Features

- **Component-based `GameObject` model** — declare game objects as plain structs with `#[base]` and `#[component]` fields; the `#[derive(GameObject)]` macro wires up the full lifecycle (`start`, `update`, `fixed_update`, `late_update`, `draw`, `destroy`, `on_message`) for you.
- **Pixel-perfect 2D renderer** — fixed logical resolution (480×270) with nearest-neighbor upscaling, sprite z-index layering, and interpolated ("blended") rendering between fixed-update steps for smooth motion at any frame rate.
- **Kinematic physics** — `Body` component with `move_and_slide`, floor/wall/ceiling detection and floor snapping, in the spirit of Godot's `CharacterBody2D`.
- **AABB collision** — box colliders with sensors/triggers and event dispatch on enter/exit.
- **Scene graph & transforms** — hierarchical `Transform2D` with parent-child position, rotation, and scale composition.
- **Scene management** — a scene stack with `push`, `pop`, `change`, and `clear`, driven by a queued command pattern.
- **Tilemaps & LDtk import** — build levels visually in [LDtk](https://ldtk.io/) and load them directly.
- **Sprite animation** — frame-based spritesheet animation.
- **Camera** — follow camera with deadzone.
- **Input** — keyboard action mapping, axis/vector helpers, and full mouse support (position, pressed/just-pressed).
- **Audio** — sound effects and music playback via `rodio`.
- **Timers** — fire-and-forget or repeating timers with event callbacks.
- **Messaging & events** — per-object mailboxes, global events, and typed inter-object messaging.
- **Built-in networking** — WebSocket client/server (via `tokio` + `tokio-tungstenite`) with binary message serialization (`bincode`), for multiplayer games.

## Quick start

Add the engine to your `Cargo.toml`:

```toml
[dependencies]
alone-engine = "0.1.0"
```

Minimal example:

```rust
use alone_engine::{App, Base, GameObject, GameObjectBase, Scene};

#[derive(GameObject)]
pub struct MainScene {
    #[base]
    base: Base,
}

impl MainScene {
    pub fn new() -> Self {
        Self {
            base: Base::default(),
        }
    }
}

impl GameObject for MainScene {
    type Message = ();
    fn start(&mut self, ctx: &mut impl alone_engine::EngineApi) {}
}

#[derive(Scene)]
pub enum GameScenes {
    MainScene(MainScene),
}

impl GameScenes {
    fn new() -> Self {
        GameScenes::MainScene(MainScene::new())
    }
}

fn main() {
    App::new(GameScenes::new()).run();
}
```

Run it:

```shell
cargo run
```

A more complete example — a character with sprite animation, kinematic movement, collision, and a follow camera — is available in [`example/`](./example).

## A slightly bigger taste

```rust
#[derive(GameObject)]
pub struct Player {
    #[base]
    base: Base,
    #[component(interface = IBody)]
    body: Body,
    #[component]
    collision: Collider,
    #[component]
    sprite: Sprite,
    #[component]
    camera: Camera,
}

impl GameObject for Player {
    type Message = ();

    fn fixed_update(&mut self, ctx: &mut impl EngineApi, delta: f32) {
        let speed = 300.0;
        let gravity = 1500.0;

        let direction = ctx.get_key_axis(KeyCode::KeyA, KeyCode::KeyD);
        self.velocity_mut().x = speed * direction;
        self.velocity_mut().y += gravity * delta;

        if self.is_on_floor() && ctx.is_key_just_pressed(KeyCode::Space, true) {
            self.velocity_mut().y = -600.0;
        }

        self.move_and_slide(ctx, delta);
    }
}
```

## Project structure

```
src/
├── core/        # Engine API traits, dispatch, LDtk import
├── runtime/     # App loop, context, world, scene stack
├── objects/     # GameObject/Scene traits, spawning, networking
├── components/  # Body, Collider, Camera, Sprite, SpriteAnimation, Tilemap, Timer, Audio
├── render/      # Software renderer (winit + pixels), draw queue
├── collision/   # AABB collision resolution
├── math/        # Vector2, Transform2D, Rect, Color
├── ui/          # (planned — not implemented yet)
├── input.rs     # Keyboard/mouse input
├── event.rs     # Messaging & global events
├── audio.rs     # Sound/music playback
└── resources.rs # Asset loading & handles

macros/          # #[derive(GameObject)] / #[derive(Scene)] proc macros
example/         # Runnable example project
```

## Design notes

- **Fixed logical resolution.** Everything is drawn onto a 480×270 buffer and scaled up with nearest-neighbor filtering, keeping pixel art crisp. This also means per-pixel effects (lighting, palette shifts, distortions) are cheap to implement in software.
- **Not an archetype ECS.** Components live as typed fields on your `GameObject` structs (closer to Godot's Node/Component composition than to Bevy-style data tables).
- **Kinematic, not rigid-body.** Physics is velocity-driven with AABB collision — great for platformers and top-down games, not intended for rotation-heavy or restitution-based physics.

## Known limitations / roadmap

- No built-in UI or text rendering yet — draw text using a bitmap-font spritesheet in the meantime.
- No particle system yet.
- No line-of-sight/raycast query API yet.
Contributions and issues are welcome.

## License

Alone Engine is licensed under the [MIT License](./LICENSE).