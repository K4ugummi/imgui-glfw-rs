//! GLFW input handling backend for [imgui-rs](https://docs.rs/imgui).
//!
//! This crate connects GLFW window events to imgui, handling mouse, keyboard,
//! scroll, clipboard, cursor icons, and HiDPI scaling. It also bundles an
//! OpenGL renderer so a single [`ImguiGLFW`] value is all you need.
//!
//! # Quick start
//!
//! ```rust,no_run
//! # use glfw::Context;
//! # use imgui::Context as ImContext;
//! # use imgui_glfw_rs::{ImguiGLFW, glfw};
//! # let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
//! # let (mut window, events) = glfw.create_window(800, 600, "", glfw::WindowMode::Windowed).unwrap();
//! # window.make_current();
//! # window.set_all_polling(true);
//! # gl::load_with(|s| window.get_proc_address(s).map_or(std::ptr::null(), |f| f as *const _));
//! let mut imgui = ImContext::create();
//! let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window).unwrap();
//!
//! while !window.should_close() {
//!     let ui = imgui_glfw.frame(&mut window, &mut imgui);
//!     ui.show_demo_window(&mut true);
//!     imgui_glfw.draw(&mut imgui, &mut window);
//!     window.swap_buffers();
//!
//!     glfw.poll_events();
//!     for (_, event) in glfw::flush_messages(&events) {
//!         let captured = imgui_glfw.handle_event(&mut imgui, &event);
//!         if captured {
//!             // imgui consumed this event; don't forward to app logic
//!         }
//!     }
//! }
//! ```
//!
//! See `examples/hello_world.rs` for a more complete example.
//! Run it with `cargo run --example hello_world`.

/// Use the re-exported glfw crate to avoid version conflicts.
pub use glfw;
/// Use the re-exported imgui crate to avoid version conflicts.
pub use imgui;

mod event_handler;

use event_handler::{handle_key, handle_key_modifier};
use glfw::ffi::{GLFWcursor, GLFWwindow};
use glfw::{Action, Window, WindowEvent};
use imgui::{BackendFlags, ConfigFlags, Context, MouseCursor};
use imgui_opengl_renderer_rs::Renderer;
pub use imgui_opengl_renderer_rs::RendererError;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;
use std::time::Instant;

struct GlfwClipboardBackend(*mut c_void);

impl imgui::ClipboardBackend for GlfwClipboardBackend {
    fn get(&mut self) -> Option<String> {
        let char_ptr = unsafe { glfw::ffi::glfwGetClipboardString(self.0 as *mut GLFWwindow) };
        if char_ptr.is_null() {
            return None;
        }
        let c_str = unsafe { CStr::from_ptr(char_ptr) };
        Some(String::from_utf8_lossy(c_str.to_bytes()).into_owned())
    }

    fn set(&mut self, value: &str) {
        if let Ok(c_string) = CString::new(value) {
            unsafe {
                glfw::ffi::glfwSetClipboardString(self.0 as *mut GLFWwindow, c_string.as_ptr());
            }
        }
    }
}

/// Map an imgui [`MouseCursor`] to the corresponding GLFW C cursor constant.
fn mouse_cursor_to_glfw(cursor: MouseCursor) -> i32 {
    match cursor {
        MouseCursor::Arrow => glfw::ffi::GLFW_ARROW_CURSOR,
        MouseCursor::TextInput => glfw::ffi::GLFW_IBEAM_CURSOR,
        MouseCursor::ResizeAll => glfw::ffi::GLFW_RESIZE_ALL_CURSOR,
        MouseCursor::ResizeNS => glfw::ffi::GLFW_RESIZE_NS_CURSOR,
        MouseCursor::ResizeEW => glfw::ffi::GLFW_RESIZE_EW_CURSOR,
        MouseCursor::ResizeNESW => glfw::ffi::GLFW_RESIZE_NESW_CURSOR,
        MouseCursor::ResizeNWSE => glfw::ffi::GLFW_RESIZE_NWSE_CURSOR,
        MouseCursor::Hand => glfw::ffi::GLFW_POINTING_HAND_CURSOR,
        MouseCursor::NotAllowed => glfw::ffi::GLFW_NOT_ALLOWED_CURSOR,
    }
}

/// Combined imgui input handler and OpenGL renderer for a GLFW window.
///
/// Create one with [`ImguiGLFW::new`], then call [`frame`](Self::frame),
/// build your UI, call [`draw`](Self::draw), and forward events with
/// [`handle_event`](Self::handle_event) each iteration of your main loop.
pub struct ImguiGLFW {
    last_frame: Instant,
    current_cursor: Option<MouseCursor>,
    /// Cached GLFW cursor pointers, indexed by [`MouseCursor`] discriminant.
    cursor_cache: [*mut GLFWcursor; MouseCursor::COUNT],
    renderer: Renderer,
}

impl ImguiGLFW {
    /// Create a new backend, initialising the clipboard, OpenGL renderer, and
    /// backend flags.
    ///
    /// `window` must be the current OpenGL context.
    ///
    /// Returns an error if the OpenGL shader/program compilation fails.
    pub fn new(imgui: &mut Context, window: &mut Window) -> Result<Self, RendererError> {
        let window_ptr = unsafe { glfw::ffi::glfwGetCurrentContext() } as *mut c_void;
        imgui.set_clipboard_backend(GlfwClipboardBackend(window_ptr));

        let io_mut = imgui.io_mut();
        io_mut.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
        io_mut.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);

        let renderer = Renderer::new(imgui, |s| {
            window
                .get_proc_address(s)
                .map_or(std::ptr::null(), |f| f as *const _)
        })?;

        // Pre-create all GLFW cursors via FFI so we get the full set
        // (including ResizeNWSE, ResizeNESW, ResizeAll, NotAllowed).
        // Some platforms don't support all shapes and report
        // GLFW_CURSOR_UNAVAILABLE. Temporarily suppress the error callback
        // so that returns null instead of panicking via glfw-rs.
        let mut cursor_cache = [std::ptr::null_mut(); MouseCursor::COUNT];
        unsafe {
            let prev_cb = glfw::ffi::glfwSetErrorCallback(None);
            for variant in MouseCursor::VARIANTS {
                let ptr = glfw::ffi::glfwCreateStandardCursor(mouse_cursor_to_glfw(variant));
                cursor_cache[variant as usize] = ptr;
            }
            glfw::ffi::glfwSetErrorCallback(prev_cb);
        }

        Ok(Self {
            last_frame: Instant::now(),
            current_cursor: None,
            cursor_cache,
            renderer,
        })
    }

    /// Handle a GLFW window event and forward it to imgui.
    ///
    /// Returns `true` if imgui wants to capture this event (i.e. the event
    /// should not be forwarded to your application logic).
    pub fn handle_event(&mut self, imgui: &mut Context, event: &WindowEvent) -> bool {
        let io = imgui.io_mut();

        match *event {
            WindowEvent::MouseButton(mouse_btn, action, _) => {
                let button = match mouse_btn {
                    glfw::MouseButton::Button1 => imgui::MouseButton::Left,
                    glfw::MouseButton::Button2 => imgui::MouseButton::Right,
                    glfw::MouseButton::Button3 => imgui::MouseButton::Middle,
                    glfw::MouseButton::Button4 => imgui::MouseButton::Extra1,
                    glfw::MouseButton::Button5 => imgui::MouseButton::Extra2,
                    _ => return false,
                };
                io.add_mouse_button_event(button, action != Action::Release);
                io.want_capture_mouse
            }
            WindowEvent::CursorPos(x, y) => {
                io.add_mouse_pos_event([x as f32, y as f32]);
                io.want_capture_mouse
            }
            WindowEvent::CursorEnter(false) => {
                io.add_mouse_pos_event([-f32::MAX, -f32::MAX]);
                false
            }
            WindowEvent::Scroll(h, v) => {
                io.add_mouse_wheel_event([h as f32, v as f32]);
                io.want_capture_mouse
            }
            WindowEvent::Char(character) => {
                io.add_input_character(character);
                io.want_capture_keyboard
            }
            WindowEvent::Key(key, _, action, modifier) => {
                handle_key_modifier(io, &modifier);
                handle_key(io, &key, action != Action::Release);
                io.want_capture_keyboard
            }
            WindowEvent::Focus(focused) => {
                io.app_focus_lost = !focused;
                false
            }
            _ => false,
        }
    }

    /// Start a new imgui frame.
    ///
    /// Updates delta time, display size, and framebuffer scale, then returns
    /// the [`Ui`](imgui::Ui) handle you use to build widgets. Call this once
    /// per iteration, before any UI code.
    pub fn frame<'a>(&mut self, window: &mut Window, imgui: &'a mut Context) -> &'a mut imgui::Ui {
        let io = imgui.io_mut();

        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;
        io.delta_time = if delta_s > 0.0 { delta_s } else { 1.0 / 60.0 };

        let window_size = window.get_size();
        io.display_size = [window_size.0 as f32, window_size.1 as f32];

        let fb_size = window.get_framebuffer_size();
        if window_size.0 > 0 && window_size.1 > 0 {
            io.display_framebuffer_scale = [
                fb_size.0 as f32 / window_size.0 as f32,
                fb_size.1 as f32 / window_size.1 as f32,
            ];
        }

        imgui.frame()
    }

    /// Finish the frame: update the mouse cursor and render the draw data.
    ///
    /// Call this after you are done building your UI for the frame and before
    /// `window.swap_buffers()`. The cursor icon is only updated when it
    /// actually changes.
    pub fn draw(&mut self, imgui: &mut Context, window: &mut Window) {
        let io = imgui.io();
        if !io
            .config_flags
            .contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE)
        {
            let window_ptr = unsafe { glfw::ffi::glfwGetCurrentContext() };
            match imgui.mouse_cursor() {
                Some(mouse_cursor) if !io.mouse_draw_cursor => {
                    if self.current_cursor != Some(mouse_cursor) {
                        window.set_cursor_mode(glfw::CursorMode::Normal);
                        let mut ptr = self.cursor_cache[mouse_cursor as usize];
                        if ptr.is_null() {
                            // Platform doesn't support this shape; fall back to Arrow.
                            ptr = self.cursor_cache[MouseCursor::Arrow as usize];
                        }
                        unsafe { glfw::ffi::glfwSetCursor(window_ptr, ptr) };
                        self.current_cursor = Some(mouse_cursor);
                    }
                }
                _ => {
                    if self.current_cursor.is_some() {
                        self.current_cursor = None;
                        window.set_cursor_mode(glfw::CursorMode::Hidden);
                    }
                }
            }
        }

        self.renderer.render(imgui);
    }
}

impl Drop for ImguiGLFW {
    fn drop(&mut self) {
        for ptr in &self.cursor_cache {
            if !ptr.is_null() {
                unsafe { glfw::ffi::glfwDestroyCursor(*ptr) };
            }
        }
    }
}
