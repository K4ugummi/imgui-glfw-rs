# imgui-glfw-rs: GLFW Input handling for ImGui
[![crates.io](https://img.shields.io/crates/v/imgui-glfw-rs.svg)](https://crates.io/crates/imgui-glfw-rs)
[![Documentation on docs.rs](https://docs.rs/imgui-glfw-rs/badge.svg)](https://docs.rs/imgui-glfw-rs)
[![Dependencies](https://deps.rs/repo/github/k4ugummi/imgui-glfw-rs/status.svg)](https://deps.rs/repo/github/k4ugummi/imgui-glfw-rs)

GLFW input handling for imgui

## How to use
```rust
// Use the reexported glfw crate to avoid version conflicts.
use imgui_glfw_rs::glfw;
// Use the reexported imgui crate to avoid version conflicts.
use imgui_glfw_rs::imgui;

use imgui_glfw_rs::ImguiGLFW;
// ImGui uses { ... }

fn main() {
    // Initialize imgui and glfw and imgui renderer.
    // { ... }

    let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window).unwrap();

    while !window.should_close() {
        let ui = imgui_glfw.frame(&mut window, &mut imgui);

        // Draw your ui.
        // { ... }

        imgui_glfw.draw(&mut imgui, &mut window);

        window.swap_buffers();

        // Handle imgui events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            let captured = imgui_glfw.handle_event(&mut imgui, &event);
            if captured {
                // imgui wants this event; skip forwarding to app logic
            }
        }
    }
}
```

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

# Compiling and running the example
```sh
git clone https://github.com/K4ugummi/imgui-glfw-rs.git
cd imgui-glfw-rs
cargo run --example hello_world
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
