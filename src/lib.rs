//! GLFW input handling backend for [imgui-rs](https://docs.rs/imgui).
//!
//! This crate connects GLFW window events to imgui, handling mouse, keyboard,
//! scroll, clipboard, cursor icons, and HiDPI scaling.
//!
//! # Features
//!
//! No features are enabled by default. Pick the renderer you need:
//!
//! - **`opengl`** — bundles the
//!   [`imgui-opengl-renderer-rs`](https://docs.rs/imgui-opengl-renderer-rs)
//!   crate so a single [`ImguiGLFW`] value handles both input and rendering.
//! - **`vulkan`** — bundles the
//!   [`imgui-vulkan-renderer-rs`](https://docs.rs/imgui-vulkan-renderer-rs)
//!   crate and re-exports it for Vulkan-based rendering.
//!
//! Without either feature, [`ImguiGLFW`] handles input only — call
//! [`update_cursors`](ImguiGLFW::update_cursors) and your own renderer each
//! frame instead of `draw`.
//!
//! # Quick start (with `opengl` feature)
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
//! # Using a custom renderer (without `opengl` feature)
//!
//! ```rust,ignore
//! // Cargo.toml: imgui-glfw-rs = { version = "..." }  (no feature flags)
//! let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);
//!
//! while !window.should_close() {
//!     let ui = imgui_glfw.frame(&mut window, &mut imgui);
//!     // ... build UI ...
//!     imgui_glfw.update_cursors(&mut imgui, &mut window);
//!     my_renderer.render(imgui.render());
//!     window.swap_buffers();
//!     // ... poll events ...
//! }
//! ```
//!
//! See `examples/hello_opengl.rs` and `examples/hello_vulkan.rs` for complete
//! examples. Run them with:
//!
//! ```sh
//! cargo run --example hello_opengl --features opengl
//! cargo run --example hello_vulkan --features vulkan
//! ```

/// Use the re-exported glfw crate to avoid version conflicts.
pub use glfw;
/// Use the re-exported imgui crate to avoid version conflicts.
pub use imgui;

mod event_handler;

use event_handler::{handle_key, handle_key_modifier};
use glfw::ffi::{GLFWcursor, GLFWwindow};
use glfw::{Action, Context as GlfwContext, Window, WindowEvent};
use imgui::{BackendFlags, ConfigFlags, Context, MouseCursor};
#[cfg(feature = "opengl")]
use imgui_opengl_renderer_rs::Renderer;
#[cfg(feature = "opengl")]
pub use imgui_opengl_renderer_rs::RendererError;
#[cfg(feature = "vulkan")]
pub use imgui_vulkan_renderer_rs;
#[cfg(feature = "vulkan")]
pub use imgui_vulkan_renderer_rs::RendererError as VulkanRendererError;
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

/// imgui input handler (and optional OpenGL renderer) for a GLFW window.
///
/// Create one with [`ImguiGLFW::new`], then call [`frame`](Self::frame),
/// build your UI, call `draw` (with the `opengl` feature) or
/// [`update_cursors`](Self::update_cursors) + your own renderer, and forward
/// events with [`handle_event`](Self::handle_event) each iteration of your
/// main loop.
pub struct ImguiGLFW {
    last_frame: Instant,
    current_cursor: Option<MouseCursor>,
    /// Cached GLFW cursor pointers, indexed by [`MouseCursor`] discriminant.
    cursor_cache: [*mut GLFWcursor; MouseCursor::COUNT],
    #[cfg(feature = "opengl")]
    renderer: Renderer,
}

/// Shared parts returned by [`init_common`].
struct CommonInit {
    cursor_cache: [*mut GLFWcursor; MouseCursor::COUNT],
}

/// Set up clipboard, backend flags, and cursor cache.
fn init_common(imgui: &mut Context, window: &Window) -> CommonInit {
    let window_ptr = window.window_ptr() as *mut c_void;
    imgui.set_clipboard_backend(GlfwClipboardBackend(window_ptr));

    let io_mut = imgui.io_mut();
    io_mut.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
    io_mut.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);

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

    CommonInit { cursor_cache }
}

impl ImguiGLFW {
    /// Create a new backend, initialising the clipboard, OpenGL renderer, and
    /// backend flags.
    ///
    /// `window` must be the current OpenGL context.
    ///
    /// Returns an error if the OpenGL shader/program compilation fails.
    #[cfg(feature = "opengl")]
    pub fn new(imgui: &mut Context, window: &mut Window) -> Result<Self, RendererError> {
        let common = init_common(imgui, window);
        let renderer = Renderer::new(imgui, |s| {
            window
                .get_proc_address(s)
                .map_or(std::ptr::null(), |f| f as *const _)
        })?;

        Ok(Self {
            last_frame: Instant::now(),
            current_cursor: None,
            cursor_cache: common.cursor_cache,
            renderer,
        })
    }

    /// Create a new input-only backend (no renderer).
    ///
    /// Use this when you bring your own renderer (Vulkan, wgpu, etc.).
    /// Call [`update_cursors`](Self::update_cursors) each frame instead of
    /// `draw`.
    #[cfg(not(feature = "opengl"))]
    pub fn new(imgui: &mut Context, window: &mut Window) -> Self {
        let common = init_common(imgui, window);
        Self {
            last_frame: Instant::now(),
            current_cursor: None,
            cursor_cache: common.cursor_cache,
        }
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

    /// Update the GLFW mouse cursor to match what imgui requests.
    ///
    /// Call this once per frame after building your UI.  When the `opengl`
    /// feature is enabled you can use `draw` instead, which calls this
    /// automatically before rendering.
    pub fn update_cursors(&mut self, imgui: &Context, window: &mut Window) {
        let io = imgui.io();
        if !io
            .config_flags
            .contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE)
        {
            let window_ptr = window.window_ptr();
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
    }

    /// Finish the frame: update the mouse cursor and render the draw data
    /// via the built-in OpenGL renderer.
    ///
    /// Call this after you are done building your UI for the frame and before
    /// `window.swap_buffers()`. The cursor icon is only updated when it
    /// actually changes.
    ///
    /// Only available with the `opengl` feature. Without it, call
    /// [`update_cursors`](Self::update_cursors) and your own renderer instead.
    #[cfg(feature = "opengl")]
    pub fn draw(&mut self, imgui: &mut Context, window: &mut Window) {
        self.update_cursors(imgui, window);
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
