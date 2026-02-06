# imgui-glfw-rs: GLFW Input handling for ImGui
[![crates.io](https://img.shields.io/crates/v/imgui-glfw-rs.svg)](https://crates.io/crates/imgui-glfw-rs)
[![Documentation on docs.rs](https://docs.rs/imgui-glfw-rs/badge.svg)](https://docs.rs/imgui-glfw-rs)
[![Dependencies](https://deps.rs/repo/github/k4ugummi/imgui-glfw-rs/status.svg)](https://deps.rs/repo/github/k4ugummi/imgui-glfw-rs)

GLFW input handling for imgui

## Features

No features are enabled by default. Pick the renderer you need:

- **`opengl`** — bundles the `imgui-opengl-renderer-rs` crate so a single
  `ImguiGLFW` value handles both input and rendering.
- **`vulkan`** — bundles the `imgui-vulkan-renderer-rs` crate and re-exports it
  for Vulkan-based rendering.

```toml
# OpenGL
imgui-glfw-rs = { version = "0.13.1", features = ["opengl"] }

# Vulkan
imgui-glfw-rs = { version = "0.13.1", features = ["vulkan"] }
```

Without either feature, `ImguiGLFW` handles input only — call `update_cursors()`
and your own renderer each frame instead of `draw()`.

## Prerequisites

You need a C compiler and CMake to build GLFW from source (handled automatically by the `glfw-sys` crate).

**Debian / Ubuntu:**
```sh
sudo apt install build-essential cmake libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl-dev
```

For the Vulkan example, also install:
```sh
sudo apt install libvulkan-dev
```

**Windows:**

Install [CMake](https://cmake.org/download/) and a C compiler (MSVC via [Visual Studio](https://visualstudio.microsoft.com/) or MinGW). For the Vulkan example, install the [Vulkan SDK](https://vulkan.lunarg.com/sdk/home).

## How to use

### OpenGL

```rust
use imgui_glfw_rs::glfw;
use imgui_glfw_rs::imgui;
use imgui_glfw_rs::ImguiGLFW;

fn main() {
    // Initialize glfw with an OpenGL context and load GL functions.
    // { ... }

    let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window).unwrap();

    while !window.should_close() {
        let ui = imgui_glfw.frame(&mut window, &mut imgui);

        // Draw your ui.
        // { ... }

        imgui_glfw.draw(&mut imgui, &mut window);
        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            imgui_glfw.handle_event(&mut imgui, &event);
        }
    }
}
```

### Vulkan

```rust
use imgui_glfw_rs::glfw;
use imgui_glfw_rs::imgui;
use imgui_glfw_rs::imgui_vulkan_renderer_rs::{Renderer, RendererCreateInfo};
use imgui_glfw_rs::ImguiGLFW;

fn main() {
    // Initialize glfw with ClientApi(NoApi) and set up Vulkan
    // (instance, device, swapchain, render pass, etc.).
    // { ... }

    let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);
    let mut renderer = Renderer::new(&mut imgui, &create_info).unwrap();

    while !window.should_close() {
        let ui = imgui_glfw.frame(&mut window, &mut imgui);

        // Draw your ui.
        // { ... }

        imgui_glfw.update_cursors(&imgui, &mut window);
        let draw_data = imgui.render();

        // Record command buffer, begin render pass, then:
        renderer.render(draw_data, command_buffer).unwrap();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            imgui_glfw.handle_event(&mut imgui, &event);
        }
    }
}
```

See `examples/hello_opengl.rs` and `examples/hello_vulkan.rs` for complete working examples.

## Current implemented things
- Mouse button press and release (event-based API)
- Cursor position movement (event-based API)
- Scroll movement (vertical and horizontal)
- Char input
- Key press and release
- Modifier handling
- Cursor icons (with change-detection optimization)
- Clipboard copying/pasting
- HiDPI / framebuffer scale support
- Window focus tracking
- Cursor enter/leave tracking
- `handle_event()` returns whether imgui captured the event

## Unimplemented things and known issues
- Gamepad / joystick input

# Compiling and running the examples
```sh
git clone https://github.com/K4ugummi/imgui-glfw-rs.git
cd imgui-glfw-rs
cargo run --example hello_opengl --features opengl

# Vulkan example (requires Vulkan SDK)
cargo run --example hello_vulkan --features vulkan
```

# Contributing
1. Make some changes
2. Run rustfmt for code style conformance
`cargo fmt`
3. Open a pull request

# Thanks to
- The [piston developers](https://github.com/PistonDevelopers) for maintaining the [glfw crate](https://github.com/PistonDevelopers/glfw-rs).
- [Gekkio](https://github.com/Gekkio) for maintaining the [imgui bindings](https://github.com/Gekkio/imgui-rs) for rust.
- You for using this crate and maybe even providing feedback
