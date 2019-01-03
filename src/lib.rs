//! ImGui input handling for Glfw.
//!
//! # Example use
//! You can run this example with `cargo run --example helloworld`
//! ```rust
//! # use glfw::Context;
//! # use imgui::{im_str, FontGlyphRange, ImFontConfig, ImGui, ImGuiCond};
//! use imgui_glfw_rs::ImguiGLFW;
//!
//! fn main() {
//!     // Initialize imgui and glfw.
//!     // { ... }
//! #     let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
//! #     glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
//! #
//! #     let (mut window, events) = glfw
//! #         .create_window(
//! #             1024,
//! #             768,
//! #             "imgui-glfw-rs example",
//! #             glfw::WindowMode::Windowed,
//! #         )
//! #         .expect("Failed to create window");
//! #
//! #     window.make_current();
//! #     window.set_framebuffer_size_polling(true);
//! #     window.set_cursor_pos_polling(true);
//! #     window.set_scroll_polling(true);
//! #     window.set_mouse_button_polling(true);
//! #     window.set_char_polling(true);
//! #     window.set_key_polling(true);
//! #
//! #     gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
//! #     unsafe {
//! #         gl::Enable(gl::BLEND);
//! #         gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
//! #         gl::Enable(gl::DEPTH_TEST);
//! #         gl::DepthFunc(gl::LESS);
//! #         gl::ClearColor(0.1, 0.1, 0.1, 1.0);
//! #     }
//! #
//! #     let mut imgui = ImGui::init();
//! #
//! #     imgui.fonts().add_default_font_with_config(
//! #         ImFontConfig::new()
//! #             .oversample_h(1)
//! #             .pixel_snap_h(true)
//! #             .size_pixels(24.),
//! #     );
//! #
//! #     imgui.fonts().add_font_with_config(
//! #         include_bytes!("../res/OpenSans-Regular.ttf"),
//! #         ImFontConfig::new()
//! #             .merge_mode(true)
//! #             .oversample_h(1)
//! #             .pixel_snap_h(true)
//! #             .size_pixels(24.)
//! #             .rasterizer_multiply(1.75),
//! #         &FontGlyphRange::japanese(),
//! #     );
//! #
//! #     imgui.set_font_global_scale(1.);
//!
//!     let mut imgui_glfw = ImguiGLFW::new(&mut imgui);
//! #
//! #     let renderer =
//! #         imgui_opengl_renderer::Renderer::new(&mut imgui, |s| window.get_proc_address(s) as _);
//!
//!     while !window.should_close() {
//! #         window.make_current();
//! #
//!         unsafe {
//!             gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
//!         }
//!
//!         let ui = imgui_glfw.frame(&mut window, &mut imgui);
//! #
//!         // Draw your ui.
//!         // { ... }
//! #         ui.window(im_str!("Hello world"))
//! #             .size((400., 0.), ImGuiCond::Once)
//! #             .build(|| {
//! #                 ui.text(im_str!("Hello world!"));
//! #                 ui.text(im_str!("こんにちは世界！"));
//! #                 ui.text(im_str!("This...is...imgui-rs!"));
//! #                 ui.separator();
//! #                 let mouse_pos = ui.imgui().mouse_pos();
//! #                 ui.text(im_str!(
//! #                     "Mouse Position: ({:.1},{:.1})",
//! #                     mouse_pos.0,
//! #                     mouse_pos.1
//! #                 ));
//! #             });
//! #
//! #         renderer.render(ui);
//! #
//! #         window.swap_buffers();
//!
//!         // Handle imgui events
//!         glfw.poll_events();
//!         for (_, event) in glfw::flush_messages(&events) {
//!             imgui_glfw.handle_event(&mut imgui, &event);
//!         }
//!     }
//! }
//! ```

use glfw::{Action, Key, Modifiers, MouseButton, StandardCursor, Window, WindowEvent};
use glfw::ffi::GLFWwindow;
use imgui::sys as imgui_sys;
use imgui::{ImGui, ImGuiKey, ImGuiMouseCursor};
use std::time::Instant;
use std::os::raw::{c_void, c_char};

pub struct ImguiGLFW {
    last_frame: Instant,
    mouse_press: [bool; 5],
    cursor_pos: (f64, f64),
    cursor: (ImGuiMouseCursor, Option<StandardCursor>),
}

impl ImguiGLFW {
    pub fn new(imgui: &mut ImGui) -> Self {
        {
            let io = unsafe { &mut *imgui_sys::igGetIO() };
            io.get_clipboard_text_fn = Some(get_clipboard_text);
            io.set_clipboard_text_fn = Some(set_clipboard_text);
            io.clipboard_user_data = std::ptr::null_mut();
        }

        {
            imgui.set_imgui_key(ImGuiKey::Tab, Key::Tab as u8);
            imgui.set_imgui_key(ImGuiKey::LeftArrow, Key::Left as u8);
            imgui.set_imgui_key(ImGuiKey::RightArrow, Key::Right as u8);
            imgui.set_imgui_key(ImGuiKey::UpArrow, Key::Up as u8);
            imgui.set_imgui_key(ImGuiKey::DownArrow, Key::Down as u8);
            imgui.set_imgui_key(ImGuiKey::PageUp, Key::PageUp as u8);
            imgui.set_imgui_key(ImGuiKey::PageDown, Key::PageDown as u8);
            imgui.set_imgui_key(ImGuiKey::Home, Key::Home as u8);
            imgui.set_imgui_key(ImGuiKey::End, Key::End as u8);
            imgui.set_imgui_key(ImGuiKey::Delete, Key::Delete as u8);
            imgui.set_imgui_key(ImGuiKey::Backspace, Key::Backspace as u8);
            imgui.set_imgui_key(ImGuiKey::Enter, Key::Enter as u8);
            imgui.set_imgui_key(ImGuiKey::Escape, Key::Escape as u8);
            imgui.set_imgui_key(ImGuiKey::A, Key::A as u8);
            imgui.set_imgui_key(ImGuiKey::C, Key::C as u8);
            imgui.set_imgui_key(ImGuiKey::V, Key::V as u8);
            imgui.set_imgui_key(ImGuiKey::X, Key::X as u8);
            imgui.set_imgui_key(ImGuiKey::Y, Key::Y as u8);
            imgui.set_imgui_key(ImGuiKey::Z, Key::Z as u8);
        }

        Self {
            last_frame: Instant::now(),
            mouse_press: [false; 5],
            cursor_pos: (0., 0.),
            cursor: (ImGuiMouseCursor::None, None),
        }
    }

    pub fn handle_event(&mut self, imgui: &mut ImGui, event: &WindowEvent) {
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
                imgui.set_mouse_down(self.mouse_press);
            }
            WindowEvent::CursorPos(w, h) => {
                imgui.set_mouse_pos(w as f32, h as f32);
                self.cursor_pos = (w, h);
            }
            WindowEvent::Scroll(_, d) => {
                imgui.set_mouse_wheel(d as f32);
            }
            WindowEvent::Char(character) => {
                imgui.add_input_character(character);
            }
            WindowEvent::Key(key, _, action, modifier) => {
                Self::set_mod(imgui, modifier);
                if action != Action::Release {
                    imgui.set_key(key as u8, true);
                } else {
                    imgui.set_key(key as u8, false);
                }
            }
            _ => {}
        }
    }

    pub fn frame<'a>(&mut self, window: &mut Window, imgui: &'a mut ImGui) -> imgui::Ui<'a> {
        let mouse_cursor = imgui.mouse_cursor();
        if imgui.mouse_draw_cursor() || mouse_cursor == ImGuiMouseCursor::None {
            self.cursor = (ImGuiMouseCursor::None, None);
            window.set_cursor(None);
        } else {
            if mouse_cursor != self.cursor.0 {
                let cursor = match mouse_cursor {
                    ImGuiMouseCursor::None => unreachable!("mouse_cursor was None!"),
                    ImGuiMouseCursor::Arrow => StandardCursor::Arrow,
                    ImGuiMouseCursor::TextInput => StandardCursor::IBeam,
                    ImGuiMouseCursor::Move => StandardCursor::Hand,
                    ImGuiMouseCursor::ResizeNS => StandardCursor::VResize,
                    ImGuiMouseCursor::ResizeEW => StandardCursor::HResize,
                    ImGuiMouseCursor::ResizeNESW => StandardCursor::Crosshair,
                    ImGuiMouseCursor::ResizeNWSE => StandardCursor::Crosshair,
                };

                window.set_cursor(Some(glfw::Cursor::standard(cursor)));
            }
        }

        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let window_size = window.get_size();
        let frame_size = imgui::FrameSize {
            logical_size: (window_size.0 as f64, window_size.1 as f64),
            hidpi_factor: 1.0,
        };
        let ui = imgui.frame(frame_size, delta_s);

        ui
    }

    fn set_mod(imgui: &mut ImGui, modifier: Modifiers) {
        imgui.set_key_ctrl(modifier.intersects(Modifiers::Control));
        imgui.set_key_alt(modifier.intersects(Modifiers::Alt));
        imgui.set_key_shift(modifier.intersects(Modifiers::Shift));
        imgui.set_key_super(modifier.intersects(Modifiers::Super));
    }
}

#[doc(hidden)]
pub extern "C" fn get_clipboard_text(_user_data: *mut c_void) -> *const c_char {
    unsafe {
        glfw::ffi::glfwGetClipboardString(_user_data as *mut GLFWwindow)
    }
}

#[doc(hidden)]
#[cfg_attr(feature = "cargo-clippy", allow(not_unsafe_ptr_arg_deref))]
pub extern "C" fn set_clipboard_text(_user_data: *mut c_void, text: *const c_char) {
    unsafe {
        glfw::ffi::glfwSetClipboardString(_user_data as *mut GLFWwindow, text);
    }
}
