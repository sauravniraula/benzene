## Benzene

Rust Vulkan rendering engine with a domain-first layout:
- `scene`: ECS scene data, bundles, and input-driven simulation
- `assets`: meshes, textures, materials, and typed asset IDs
- `render`: renderer, frame resources, and Vulkan internals
- `engine`: high-level entrypoint that ties assets and rendering together

The old `core` / `vulkan_backend` split and thin `V*` wrapper surface are no longer the public architecture. Renderer internals now use `ash::Instance`, `ash::Device`, and raw `vk::*` handles directly.

### Highlights
- Vulkan via `ash`
- `winit` windowing and event loop integration
- ECS scene with typed bundles such as `CameraBundle` and `MeshInstanceBundle`
- Typed asset IDs for meshes, textures, and materials
- Material binding driven by real `Texture` / `Material` resources instead of generic handle wrappers

### Requirements
- Rust stable
- Vulkan runtime / graphics driver
- Linux or Windows with Vulkan-capable GPU
- Optional for shader rebuilds: `glslc` or `shaderc-tools`

### Minimal Usage
```rust
use benzene::{
    Camera, CameraBundle, DirectionalLight, DirectionalLightBundle, Engine, MeshInstance,
    MeshInstanceBundle, Scene, Transform,
};
use nalgebra::{Vector3, Vector4};

fn build_scene(engine: &mut Engine) -> benzene::Result<Scene> {
    let mut scene = engine.create_scene();

    let texture = engine.load_texture("assets/textures/cracked-dirt512x512.jpg")?;
    let material = engine.create_material(texture)?;
    let mesh = engine.load_mesh_obj("assets/models/plane.obj")?;

    let camera = scene.spawn_camera(
        CameraBundle::new(
            Transform::new(
                Vector3::new(0.0, 2.8, 6.5),
                Vector3::new((-12.0f32).to_radians(), 0.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0),
            ),
            Camera::default(),
        )
        .named("Camera"),
    );
    scene.set_active_camera(camera);

    scene.spawn_directional_light(
        DirectionalLightBundle::new(
            Transform::identity(),
            DirectionalLight::new(Vector4::new(1.0, 1.0, 1.0, 0.2)),
        )
        .named("Sun"),
    );

    scene.spawn_mesh_instance(
        MeshInstanceBundle::new(
            Transform::new(
                Vector3::new(0.0, 0.0, -5.0),
                Vector3::zeros(),
                Vector3::new(4.0, 1.0, 4.0),
            ),
            MeshInstance::new(mesh, Some(material)),
        )
        .named("Ground"),
    );

    Ok(scene)
}
```

See [src/main.rs](/home/viristo/prog/benzene/src/main.rs) for the full `winit` sample app.

### Project Layout
```text
src/
├── assets/      # Mesh / texture / material resources and typed IDs
├── render/      # Renderer and Vulkan internals
├── scene/       # ECS scene model, bundles, input helpers
├── engine.rs    # High-level engine entrypoint
└── main.rs      # Sample application
```

### Run
```bash
cargo run
```

### Shaders
Compiled SPIR-V shaders are expected under `compiled/shaders/`, mirroring `assets/shaders/`.

If you edit GLSL source, rebuild them with:
```bash
./compile_shaders.sh
```
