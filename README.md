## benzene

Rust Vulkan engine for simple real‑time rendering. Minimal, fast, and hackable.

### Highlights
- **Vulkan via ash**: modern graphics pipeline and synchronization
- **GLFW windowing**: surface creation and event loop
- **Basic scene**: primitives (cube/plane) and a simple camera
- **GLSL → SPIR‑V**: compile script for shaders in `assets/shaders`
- **Modular design**: clear split between core engine and Vulkan backend

### Requirements
- Rust (stable)
- Vulkan runtime/driver; optional Vulkan SDK for `glslc`
- Linux/Windows with a Vulkan‑capable GPU

### Quick start
1. Compile shaders
   ```bash
   ./compile_shaders.sh
   ```
2. Run
   ```bash
   cargo run --release
   ```
3. Exit with ESC

### Project layout
```
benzene/
├── assets/
│   └── shaders/          # GLSL sources and compiled .spv
├── src/
│   ├── core/             # engine, scene, resources, rendering
│   ├── vulkan_backend/   # instance/device/swapchain/pipeline/rendering
│   ├── window/           # window config + wrapper
│   ├── shared/           # helpers (e.g., file I/O)
│   └── main.rs
├── compile_shaders.sh
└── Cargo.toml
```

### Development notes
- Edit GLSL in `assets/shaders` and re‑run `./compile_shaders.sh`
- Use `cargo run --release` for better frame times

### Troubleshooting
- Shader compile errors: install the Vulkan SDK to get `glslc` in PATH
- Vulkan loader/device errors: update GPU drivers and install the Vulkan loader