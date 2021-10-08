//! ImGui input handling for Glfw.
//!
//! # Example use
//! You can run this example with `cargo run --example hello_world`
//!
//! ```rust
//! use glfw::Context;
//! use imgui::Context as ImContext;
//! use imgui_glfw_rs::glfw;
//! use imgui_glfw_rs::imgui;
//! use imgui_glfw_rs::ImguiGLFW;
//!
//! fn main() {
//!     let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
//!     glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
//!
//!     let (mut window, events) = glfw
//!         .create_window(
//!             1024,
//!             768,
//!             "imgui-glfw-rs example",
//!             glfw::WindowMode::Windowed,
//!         )
//!         .expect("Failed to create window");
//!
//!     window.make_current();
//!     window.set_all_polling(true);
//!
//!     gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
//!     unsafe {
//!         gl::Enable(gl::BLEND);
//!         gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
//!         gl::Enable(gl::DEPTH_TEST);
//!         gl::DepthFunc(gl::LESS);
//!         gl::ClearColor(0.1, 0.1, 0.1, 1.0);
//!     }
//!
//!     let mut imgui = ImContext::create();
//!
//!     let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);
//!
//!     while !window.should_close() {
//!         unsafe {
//!             gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
//!         }
//!
//!         let ui = imgui_glfw.frame(&mut window, &mut imgui);
//!
//!         ui.show_demo_window(&mut true);
//!
//!         imgui_glfw.draw(ui, &mut window);
//!
//!         window.swap_buffers();
//!
//!         glfw.poll_events();
//!         for (_, event) in glfw::flush_messages(&events) {
//!             imgui_glfw.handle_event(&mut imgui, &event);
//!         }
//!     }
//! }
//! ```

/// Use the reexported glfw crate to avoid version conflicts.
pub use glfw;
/// Use the reexported imgui crate to avoid version conflicts.
pub use imgui;

use glfw::ffi::GLFWwindow;
use glfw::{Action, Key, Modifiers, MouseButton, StandardCursor, Window, WindowEvent};
use imgui::{ConfigFlags, Context, Key as ImGuiKey, MouseCursor, Ui};
use imgui_opengl_renderer::Renderer;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::time::Instant;

struct GlfwClipboardBackend(*mut c_void);

impl imgui::ClipboardBackend for GlfwClipboardBackend {
    fn get(&mut self) -> Option<imgui::ImString> {
        let char_ptr = unsafe { glfw::ffi::glfwGetClipboardString(self.0 as *mut GLFWwindow) };
        let c_str = unsafe { CStr::from_ptr(char_ptr) };
        Some(imgui::ImString::new(c_str.to_str().unwrap()))
    }

    fn set(&mut self, value: &imgui::ImStr) {
        unsafe {
            glfw::ffi::glfwSetClipboardString(
                self.0 as *mut GLFWwindow,
                value.as_ptr(),
            );
        };
    }
}

pub struct ImguiGLFW {
    last_frame: Instant,
    mouse_press: [bool; 5],
    cursor_pos: (f64, f64),
    cursor: (MouseCursor, Option<StandardCursor>),

    renderer: Renderer,
}

impl ImguiGLFW {
    pub fn new(imgui: &mut Context, window: &mut Window) -> Self {
        unsafe {
            let window_ptr = glfw::ffi::glfwGetCurrentContext() as *mut c_void;
            imgui.set_clipboard_backend(Box::new(GlfwClipboardBackend(window_ptr)));
        }

        let mut io_mut = imgui.io_mut();
        io_mut.key_map[ImGuiKey::Tab as usize] = Key::Tab as u32;
        io_mut.key_map[ImGuiKey::LeftArrow as usize] = Key::Left as u32;
        io_mut.key_map[ImGuiKey::RightArrow as usize] = Key::Right as u32;
        io_mut.key_map[ImGuiKey::UpArrow as usize] = Key::Up as u32;
        io_mut.key_map[ImGuiKey::DownArrow as usize] = Key::Down as u32;
        io_mut.key_map[ImGuiKey::PageUp as usize] = Key::PageUp as u32;
        io_mut.key_map[ImGuiKey::PageDown as usize] = Key::PageDown as u32;
        io_mut.key_map[ImGuiKey::Home as usize] = Key::Home as u32;
        io_mut.key_map[ImGuiKey::End as usize] = Key::End as u32;
        io_mut.key_map[ImGuiKey::Insert as usize] = Key::Insert as u32;
        io_mut.key_map[ImGuiKey::Delete as usize] = Key::Delete as u32;
        io_mut.key_map[ImGuiKey::Backspace as usize] = Key::Backspace as u32;
        io_mut.key_map[ImGuiKey::Space as usize] = Key::Space as u32;
        io_mut.key_map[ImGuiKey::Enter as usize] = Key::Enter as u32;
        io_mut.key_map[ImGuiKey::Escape as usize] = Key::Escape as u32;
        io_mut.key_map[ImGuiKey::A as usize] = Key::A as u32;
        io_mut.key_map[ImGuiKey::C as usize] = Key::C as u32;
        io_mut.key_map[ImGuiKey::V as usize] = Key::V as u32;
        io_mut.key_map[ImGuiKey::X as usize] = Key::X as u32;
        io_mut.key_map[ImGuiKey::Y as usize] = Key::Y as u32;
        io_mut.key_map[ImGuiKey::Z as usize] = Key::Z as u32;

        let renderer = Renderer::new(imgui, |s| window.get_proc_address(s) as _);

        Self {
            last_frame: Instant::now(),
            mouse_press: [false; 5],
            cursor_pos: (0., 0.),
            cursor: (MouseCursor::Arrow, None),

            renderer,
        }
    }

    pub fn handle_event(&mut self, imgui: &mut Context, event: &WindowEvent) {
        match *event {
            WindowEvent::MouseButton(mouse_btn, action, _) => {
                let index = match mouse_btn {
                    MouseButton::Button1 => 0,
                    MouseButton::Button2 => 1,
                    MouseButton::Button3 => 2,
                    MouseButton::Button4 => 3,
                    MouseButton::Button5 => 4,
                    _ => 0,
                };
                let press = action != Action::Release;
                self.mouse_press[index] = press;
                imgui.io_mut().mouse_down = self.mouse_press;
            }
            WindowEvent::CursorPos(w, h) => {
                imgui.io_mut().mouse_pos = [w as f32, h as f32];
                self.cursor_pos = (w, h);
            }
            WindowEvent::Scroll(_, d) => {
                imgui.io_mut().mouse_wheel = d as f32;
            }
            WindowEvent::Char(character) => {
                imgui.io_mut().add_input_character(character);
            }
            WindowEvent::Key(key, _, action, modifier) => {
                Self::set_mod(imgui, modifier);
                imgui.io_mut().keys_down[key as usize] = action != Action::Release;
            }
            _ => {}
        }
    }

    pub fn frame<'a>(&mut self, window: &mut Window, imgui: &'a mut Context) -> imgui::Ui<'a> {
        let io = imgui.io_mut();

        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;
        io.delta_time = delta_s;

        let window_size = window.get_size();
        io.display_size = [window_size.0 as f32, window_size.1 as f32];

        imgui.frame()
    }

    pub fn draw<'ui>(&mut self, ui: Ui<'ui>, window: &mut Window) {
        let io = ui.io();
        if !io
            .config_flags
            .contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE)
        {
            match ui.mouse_cursor() {
                Some(mouse_cursor) if !io.mouse_draw_cursor => {
                    window.set_cursor_mode(glfw::CursorMode::Normal);

                    let cursor = match mouse_cursor {
                        MouseCursor::TextInput => StandardCursor::IBeam,
                        MouseCursor::ResizeNS => StandardCursor::VResize,
                        MouseCursor::ResizeEW => StandardCursor::HResize,
                        MouseCursor::Hand => StandardCursor::Hand,
                        _ => StandardCursor::Arrow,
                    };
                    window.set_cursor(Some(glfw::Cursor::standard(cursor)));

                    if self.cursor.1 != Some(cursor) {
                        self.cursor.1 = Some(cursor);
                        self.cursor.0 = mouse_cursor;
                    }
                }
                _ => {
                    self.cursor.0 = MouseCursor::Arrow;
                    self.cursor.1 = None;
                    window.set_cursor_mode(glfw::CursorMode::Hidden);
                }
            }
        }

        self.renderer.render(ui);
    }

    fn set_mod(imgui: &mut Context, modifier: Modifiers) {
        imgui.io_mut().key_ctrl = modifier.intersects(Modifiers::Control);
        imgui.io_mut().key_alt = modifier.intersects(Modifiers::Alt);
        imgui.io_mut().key_shift = modifier.intersects(Modifiers::Shift);
        imgui.io_mut().key_super = modifier.intersects(Modifiers::Super);
    }
}
