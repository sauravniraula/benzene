## Benzene

Rust Vulkan game engine for simple real‑time rendering. Minimal surface area, ergonomic scene API, and GLFW windowing.

### Highlights
- **Vulkan via ash**: explicit, modern graphics
- **GLFW windowing**: cross‑platform surface + event loop
- **Scene + ECS**: add entities and components (camera, lights, transforms, structures)
- **Models**: load `.obj` files as `Structure3D`
- **Modular**: clear split between engine and Vulkan backend

### Requirements
- Rust (stable)
- Vulkan runtime/driver installed
- Linux/Windows with a Vulkan‑capable GPU
 - Optional (for shader rebuilds): Vulkan SDK providing `glslc` or `shaderc-tools`

### Install
Use in your workspace (path or git), or build/run this repo directly.

```toml
[dependencies]
# Local path (same workspace/monorepo)
benzene = { path = "../benzene" }

# Or from a git repo (example)
# benzene = { git = "https://github.com/<you>/benzene", tag = "v0.1.0" }
```

### Usage
Create an engine and scene, add entities and components, then run:

```rust
use benzene::core::GameEngine;
use benzene::core::ecs::{
    components::{Camera3D, PointLight3D, Structure3D, Transform3D},
    entities::game_object::GameObject,
};
use nalgebra::{Vector3, Vector4};

fn main() {
    let mut engine = GameEngine::new();
    let mut scene = engine.create_scene();

    // Camera
    let camera = GameObject::new("Camera");
    scene.add_game_object(camera.clone());
    scene.add_camera_3d_component(&camera, Camera3D::new_default());
    scene.set_active_camera(&camera);

    // Point light
    let light = GameObject::new("Light");
    scene.add_game_object(light.clone());
    scene.add_transform_3d_component(
        &light,
        Transform3D::new(
            Vector3::new(2.0, 2.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    scene.add_point_light_3d_component(&light, PointLight3D::new(Vector4::new(1.0, 1.0, 1.0, 10.0)));

    // Model from .obj
    let vase_entity = GameObject::new("Vase");
    let vase: Structure3D = engine.get_structure_from_obj("assets/models/vase-smooth.obj");
    scene.add_game_object(vase_entity.clone());
    scene.add_transform_3d_component(
        &vase_entity,
        Transform3D::new(
            Vector3::new(0.0, 0.0, -5.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    scene.add_structure_3d_component(&vase_entity, vase);

    engine.set_active_scene(scene);
    engine.run();
    engine.destroy();
}
```

### Controls (defaults)
- **W/A/S/D**: move left/right/back/forward
- **Space**: move up
- **Alt**: move down
- **Arrow keys**: look around (yaw/pitch)
- **Esc**: quit

### Public API surface (essentials)
- **Engine**
  - `GameEngine::new()`
  - `GameEngine::create_scene()` → `Scene`
  - `GameEngine::set_active_scene(Scene)`
  - `GameEngine::run()` / `GameEngine::destroy()`
  - `GameEngine::get_structure_from_obj(path)` → `Structure3D`
- **Scene**
  - `Scene::add_game_object(GameObject)`
  - `Scene::add_transform_3d_component(&GameObject, Transform3D)`
  - `Scene::add_camera_3d_component(&GameObject, Camera3D)` / `Scene::set_active_camera(&GameObject)`
  - `Scene::add_point_light_3d_component(&GameObject, PointLight3D)`
  - `Scene::add_structure_3d_component(&GameObject, Structure3D)`
- **GameObject**
  - lightweight entity handle (ID + name); attach components via `Scene`

### Assets
- `.obj` meshes are supported via `tobj`. Provide paths relative to your app.
- A default texture is bound; per‑object materials are a work‑in‑progress.

### Run
```bash
cargo run --release
```

### Shaders
Precompiled SPIR‑V shaders are included in `assets/shaders/*.spv`. If you edit GLSL source, rebuild:

```bash
./compile_shaders.sh
```

### Project layout
```
benzene/
├── assets/               # sample assets, shaders, textures
├── src/
│   ├── core/             # engine, scene, ecs, gpu helpers
│   ├── vulkan_backend/   # instance/device/swapchain/pipeline/rendering
│   ├── window/           # window config + wrapper
│   └── shared/           # helpers (e.g., file I/O)
└── Cargo.toml
```

### Troubleshooting
- If the Vulkan loader/device fails to initialize, update GPU drivers and ensure the Vulkan loader is installed.
- Use `--release` for stable frame times.
