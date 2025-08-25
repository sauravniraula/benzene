## Benzene

Rust Vulkan game engine for simple real‑time rendering. Minimal surface area, ergonomic scene API, and GLFW windowing.

### Highlights
- **Vulkan via ash**: explicit, modern graphics
- **GLFW windowing**: cross‑platform surface + event loop
- **Scene + camera**: plug a camera, add game objects, render
- **Models**: load `.obj` files or use built‑in primitives
- **Modular**: clear split between engine and Vulkan backend

### Requirements
- Rust (stable)
- Vulkan runtime/driver installed
- Linux/Windows with a Vulkan‑capable GPU

### Install
Add `benzene` as a dependency in your app:

```toml
[dependencies]
# Local path (same workspace/monorepo)
benzene = { path = "../benzene" }

# Or from a git repo (example)
# benzene = { git = "https://github.com/<you>/benzene", tag = "v0.1.0" }
```

### Usage
Create an engine, a scene, attach a camera, add objects, then run:

```rust
use benzene::core::{GameEngine, camera::Camera, primitives::plane::Plane};
use nalgebra::Vector3;

fn main() {
    let mut engine = GameEngine::new();

    let mut scene = engine.create_scene();
    scene.attach_camera(Camera::new());

    // Load models from .obj files
    let smooth_vase = engine.get_game_object_from_obj("assets/models/vase-smooth.obj");
    let mut flat_vase = engine.get_game_object_from_obj("assets/models/vase-flat.obj");
    let mut flat_torus = engine.get_game_object_from_obj("assets/models/torus-flat.obj");
    let mut smooth_torus = engine.get_game_object_from_obj("assets/models/torus-smooth.obj");

    // Built‑in primitive via a model builder
    let plane = engine.get_game_object_from_model_builder::<Plane>();

    // Position objects in world space
    flat_vase.set_position(Vector3::new(-2.0, 0.0, 0.0));
    flat_torus.set_position(Vector3::new(4.0, 0.5, 4.0));
    smooth_torus.set_position(Vector3::new(4.0, 0.5, 0.0));

    // Add them to the scene
    scene.add_game_object(smooth_vase);
    scene.add_game_object(plane);
    scene.add_game_object(flat_vase);
    scene.add_game_object(flat_torus);
    scene.add_game_object(smooth_torus);

    engine.set_active_scene(scene);
    engine.run();
    engine.destroy();
}
```

### Controls (defaults)
- **W/A/S/D**: move left/right/back/forward
- **Space**: move up
- **Alt**: move down
- **Shift**: speed boost
- **Arrow keys**: look around (yaw/pitch)
- **Esc**: quit

### Public API surface (essentials)
- **Engine**
  - `GameEngine::new()`
  - `GameEngine::create_scene()` → `Scene`
  - `GameEngine::set_active_scene(Scene)`
  - `GameEngine::run()` / `GameEngine::destroy()`
  - `GameEngine::get_game_object_from_obj(path)`
  - `GameEngine::get_game_object_from_model_builder::<T: ModelBuilder>()`
- **Scene**
  - `Scene::attach_camera(Camera)` / `Scene::detach_camera()`
  - `Scene::add_game_object(GameObject)`
- **GameObject**
  - transforms like `set_position(Vector3)`, and participates in rendering

### Assets
- `.obj` meshes are supported via `tobj`. Provide paths relative to your app.
- A default texture is bound; per‑object materials are a work‑in‑progress.

### Run
```bash
cargo run --release
```

### Project layout
```
benzene/
├── assets/               # sample assets used by examples/tests
├── src/
│   ├── core/             # engine, scene, camera, primitives
│   ├── vulkan_backend/   # instance/device/swapchain/pipeline/rendering
│   ├── window/           # window config + wrapper
│   └── shared/           # helpers (e.g., file I/O)
└── Cargo.toml
```

### Troubleshooting
- If the Vulkan loader/device fails to initialize, update GPU drivers and ensure the Vulkan loader is installed.
- Use `--release` for stable frame times.
