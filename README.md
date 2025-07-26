# Vulkan Engine

A Rust-based Vulkan graphics engine that demonstrates the complete Vulkan initialization pipeline, from instance creation to rendering a triangle.

## Project Structure

```
vulkan_engine/
├── Cargo.toml
├── compile_shaders.sh
└── src/
    ├── main.rs              # Application entry point
    ├── lib.rs               # Library module declarations
    ├── entities/            # Core Vulkan entities
    │   ├── mod.rs
    │   ├── window.rs        # GLFW window management
    │   ├── window_instruction.rs
    │   ├── vulkan_app.rs    # Main Vulkan application
    │   └── compute_device.rs # Physical device selection
    ├── utils/               # Utility functions
    │   ├── mod.rs
    │   └── debug_callback.rs
    ├── macros/              # Custom macros
    │   ├── mod.rs
    │   └── print.rs
    ├── components/          # (Empty - for future use)
    │   └── mod.rs
    └── shaders/             # GLSL shaders
        ├── shader.vert
        ├── shader.vert.spv
        ├── shader.frag
        └── shader.frag.spv
```

## Dependencies

### External Dependencies
- **ash**: Vulkan bindings for Rust
- **glfw**: Window management and surface creation

### Internal Dependencies

```mermaid
graph TD
    A[main.rs] --> B[lib.rs]
    B --> C[entities/mod.rs]
    B --> D[utils/mod.rs]
    B --> E[macros/mod.rs]
    B --> F[components/mod.rs]
    
    C --> G[window.rs]
    C --> H[window_instruction.rs]
    C --> I[vulkan_app.rs]
    C --> J[compute_device.rs]
    
    D --> K[debug_callback.rs]
    E --> L[print.rs]
    
    I --> G
    I --> J
    I --> K
    I --> M[print_separator macro]
    
    J --> G
    K --> M
```

## Function Call Flowcharts

### 1. Application Initialization Flow

```mermaid
flowchart TD
    A[main] --> B[Window::new]
    B --> C[glfw::init]
    B --> D[glfw::create_window]
    B --> E[Window struct created]
    
    A --> F[VulkanApp::new]
    F --> G[Entry::linked]
    F --> H[Create ApplicationInfo]
    F --> I[Get GLFW extensions]
    F --> J[Enumerate layers]
    F --> K[Enumerate extensions]
    F --> L[Create Vulkan instance]
    F --> M[Create debug messenger]
    F --> N[Create surface]
    F --> O[Enumerate physical devices]
    F --> P[ComputeDevice::new for each device]
    F --> Q[ComputeDevice::select_device_and_queue]
    F --> R[ComputeDevice::select_present_queue]
    F --> S[Create logical device]
    F --> T[Get graphics and present queues]
    F --> U[Create swapchain]
    F --> V[Create image views]
    F --> W[Load shaders]
    F --> X[Create pipeline]
    F --> Y[Create framebuffers]
    F --> Z[Create command pool and buffers]
    F --> AA[Create synchronization objects]
    
    A --> BB[Window::start]
    BB --> CC[Render loop]
    CC --> DD[VulkanApp::draw_frame]
    DD --> EE[Wait for fence]
    DD --> FF[Acquire next image]
    DD --> GG[Record command buffer]
    DD --> HH[Submit to graphics queue]
    DD --> II[Present to surface]
```

### 2. Window Management Flow

```mermaid
flowchart TD
    A[Window::new] --> B[glfw::init]
    B --> C[Set window hints]
    C --> D[glfw::create_window]
    D --> E[Window struct]
    
    F[Window::get_required_glfw_extensions] --> G[glfw::get_required_instance_extensions]
    
    H[Window::get_surface] --> I[window::create_window_surface]
    I --> J[vk::SurfaceKHR]
    
    K[Window::get_framebuffer_size] --> L[window::get_framebuffer_size]
    
    M[Window::start] --> N[while !should_close]
    N --> O[on_render_loop]
    N --> P[glfw::poll_events]
    N --> Q[Process events]
    Q --> R[events_handler]
    R --> S[WindowInstruction::Close]
    R --> T[WindowInstruction::None]
```

### 3. Physical Device Selection Flow

```mermaid
flowchart TD
    A[ComputeDevice::new] --> B[Store device properties]
    B --> C[Store queue properties]
    B --> D[Store supported extensions]
    B --> E[Store surface capabilities]
    B --> F[Store surface formats]
    B --> G[Store present modes]
    
    H[ComputeDevice::select_device_and_queue] --> I[Iterate devices]
    I --> J{Is discrete GPU?}
    J -->|No| I
    J -->|Yes| K{Supports swapchain?}
    K -->|No| I
    K -->|Yes| L{Has surface formats?}
    L -->|No| I
    L -->|Yes| M{Has present modes?}
    M -->|No| I
    M -->|Yes| N{Has graphics queue?}
    N -->|No| I
    N -->|Yes| O[Return device and queue index]
    
    P[ComputeDevice::select_present_queue] --> Q[Iterate queue families]
    Q --> R{Supports surface?}
    R -->|No| Q
    R -->|Yes| S[Return queue index]
    
    T[ComputeDevice::select_surface_format] --> U{Preferred format?}
    U -->|Yes| V[Return preferred format]
    U -->|No| W[Return first format]
    
    X[ComputeDevice::select_present_mode] --> Y{Supports mailbox?}
    Y -->|Yes| Z[Return mailbox mode]
    Y -->|No| AA[Return FIFO mode]
    
    BB[ComputeDevice::select_image_extent] --> CC[Get window size]
    CC --> DD[Clamp to surface capabilities]
    DD --> EE[Return extent]
    
    FF[ComputeDevice::select_swapchain_image_count] --> GG[Get min count]
    GG --> HH{Can add more?}
    HH -->|Yes| II[Add one]
    HH -->|No| JJ[Return min count]
```

### 4. Vulkan App Initialization Flow

```mermaid
flowchart TD
    A[VulkanApp::new] --> B[Create Entry]
    B --> C[Create ApplicationInfo]
    C --> D[Get GLFW extensions]
    D --> E[Enumerate layers]
    E --> F[Enumerate extensions]
    F --> G[Create instance]
    G --> H[Create debug messenger]
    H --> I[Create surface]
    I --> J[Enumerate physical devices]
    J --> K[Create ComputeDevice for each]
    K --> L[Select best device]
    L --> M[Create logical device]
    M --> N[Get queues]
    N --> O[Create swapchain]
    O --> P[Get swapchain images]
    P --> Q[Create image views]
    Q --> R[Load shaders]
    R --> S[Create pipeline]
    S --> T[Create render pass]
    T --> U[Create framebuffers]
    U --> V[Create command pool]
    V --> W[Allocate command buffers]
    W --> X[Create synchronization objects]
```

### 5. Rendering Pipeline Flow

```mermaid
flowchart TD
    A[VulkanApp::draw_frame] --> B[Wait for fence]
    B --> C[Reset fence]
    C --> D[Acquire next image]
    D --> E[Reset command buffer]
    E --> F[Record command buffer]
    F --> G[Submit to graphics queue]
    G --> H[Present to surface]
    
    I[record_command_buffer] --> J[Begin command buffer]
    J --> K[Begin render pass]
    K --> L[Bind pipeline]
    L --> M[Set viewport]
    M --> N[Set scissor]
    N --> O[Draw]
    O --> P[End render pass]
    P --> Q[End command buffer]
```

### 6. Debug Callback Flow

```mermaid
flowchart TD
    A[vulkan_debug_callback] --> B[Print separator]
    B --> C[Extract message data]
    C --> D[Print message type]
    D --> E[Print severity]
    E --> F[Print message]
    F --> G[Return FALSE]
    
    H[print_separator macro] --> I[print_separator_fn]
    I --> J[Calculate separator length]
    J --> K[Create left separator]
    K --> L[Create right separator]
    L --> M[Print formatted separator]
```

## Class Dependencies

### VulkanApp Dependencies
```mermaid
graph TD
    A[VulkanApp] --> B[Window]
    A --> C[ComputeDevice]
    A --> D[debug_utils::Instance]
    A --> E[surface::Instance]
    A --> F[swapchain::Device]
    A --> G[vulkan_debug_callback]
    A --> H[print_separator macro]
    
    B --> I[glfw]
    C --> B
    C --> E
    G --> H
```

### Window Dependencies
```mermaid
graph TD
    A[Window] --> B[glfw::Glfw]
    A --> C[glfw::PWindow]
    A --> D[glfw::GlfwReceiver]
    A --> E[WindowInstruction]
    A --> F[ash::vk]
```

### ComputeDevice Dependencies
```mermaid
graph TD
    A[ComputeDevice] --> B[ash::vk]
    A --> C[ash::khr::surface]
    A --> D[Window]
    A --> E[std::ffi::CString]
```

## Function Call Hierarchy

### Main Entry Point
```
main()
├── Window::new()
│   ├── glfw::init()
│   ├── glfw::create_window()
│   └── Window struct
├── VulkanApp::new()
│   ├── Entry::linked()
│   ├── Create ApplicationInfo
│   ├── Window::get_required_glfw_extensions()
│   ├── Enumerate layers/extensions
│   ├── Create Vulkan instance
│   ├── Create debug messenger
│   ├── Window::get_surface()
│   ├── Enumerate physical devices
│   ├── ComputeDevice::new() for each device
│   ├── ComputeDevice::select_device_and_queue()
│   ├── ComputeDevice::select_present_queue()
│   ├── Create logical device
│   ├── Get queues
│   ├── Create swapchain
│   ├── Create image views
│   ├── Load shaders
│   ├── Create pipeline
│   ├── Create framebuffers
│   ├── Create command pool/buffers
│   └── Create synchronization objects
└── Window::start()
    ├── Render loop
    │   └── VulkanApp::draw_frame()
    │       ├── Wait for fence
    │       ├── Acquire next image
    │       ├── record_command_buffer()
    │       ├── Submit to queue
    │       └── Present
    └── Event handling
```

## Key Features

1. **Complete Vulkan Initialization**: From instance creation to rendering pipeline
2. **Physical Device Selection**: Automatic selection of best available GPU
3. **Surface Management**: GLFW integration for window surface creation
4. **Debug Support**: Vulkan validation layers with custom debug callbacks
5. **Synchronization**: Proper fence and semaphore management
6. **Triangle Rendering**: Basic graphics pipeline with vertex and fragment shaders

## Building and Running

1. **Compile shaders**:
   ```bash
   ./compile_shaders.sh
   ```

2. **Build and run**:
   ```bash
   cargo run
   ```

3. **Exit**: Press ESC key to close the application

## Dependencies Summary

- **ash**: Vulkan API bindings
- **glfw**: Window management and surface creation
- **Custom macros**: Debug output formatting
- **GLSL shaders**: Vertex and fragment shaders for triangle rendering

The engine demonstrates a complete Vulkan graphics pipeline from initialization to rendering, with proper resource management and synchronization. 