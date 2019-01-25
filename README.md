# imgui-glfw-rs: GLFW Input handling for ImGui
**EXPERIMENTAL!**  
[![crates.io](https://meritbadge.herokuapp.com/imgui-glfw-rs)](https://crates.io/crates/imgui-glfw-rs)
[![Documentation on docs.rs](https://docs.rs/imgui-glfw-rs/badge.svg)](https://docs.rs/imgui)
[![Dependencies](https://deps.rs/repo/github/k4ugummi/imgui-glfw-rs/status.svg)](https://deps.rs/repo/github/k4ugummi/imgui-glfw-rs)

GLFW input handling for imgui

## How to use
```rust
use imgui_glfw_rs::glfw as glfw;    // avoid version conflicts
use imgui_glfw_rs::ImguiGLFW;
// ImGui uses { ... }

fn main() {
    // Initialize imgui and glfw and imgui renderer.
    // { ... }

    let mut imgui_glfw = ImguiGLFW::new(&mut imgui);

    while !window.should_close() {
        let ui = imgui_glfw.frame(&mut window, &mut imgui);

        // Draw your ui.
        // { ... }

        window.swap_buffers();

        // Handle imgui events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            imgui_glfw.handle_event(&mut imgui, &event);
        }
    }
}
```

## Current implemented things
- MouseButton press and release
- CursorPos movement
- Scroll movement
- Char input
- Key press and release
- Modifier handling
- Cursor icons
- Clipboard copying/pasting

## Unimplemented things and known issues

# Compiling and running the example
```sh
git clone https://github.com/K4ugummi/imgui-glfw-rs.git
cd imgui-glfw-rs
cargo run --example helloworld
```

# Contributing
1. Make some changes
2. Run rustfmt for code style conformance  
`cargo fmt`
3. Open a pull request

# Thanks to
- The piston developers for maintaining the `glfw` crate
- You for using this crate and maybe even providing feedback
