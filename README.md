# imgui-glfw-rs: GLFW Input handling for ImGui
**EXPERIMENTAL!**  
[![crates.io](https://meritbadge.herokuapp.com/imgui-glfw-rs)](https://crates.io/crates/imgui-glfw-rs)
[![Documentation on docs.rs](https://docs.rs/imgui-glfw-rs/badge.svg)](https://docs.rs/imgui)

GLFW input handling for imgui

## How to use
```rust
// ImGui uses { ... }
use imgui_glfw_rs::ImguiGLFW;

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

## Unimplemented things and known issues
- Clipboard copying/pasting crashes

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
