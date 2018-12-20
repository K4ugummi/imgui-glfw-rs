//use sdl2::sys as sdl2_sys;
use imgui::sys as imgui_sys;

//use sdl2::video::Window;
//use sdl2::EventPump;
//use sdl2::mouse::{Cursor,SystemCursor};
use glfw::{
    Action, Cursor, FlushedMessages, Key, Modifiers, MouseButton, StandardCursor, Window,
    WindowEvent,
};
use imgui::{ImGui, ImGuiKey, ImGuiMouseCursor};
use std::os::raw::{c_char, c_void};
use std::time::Instant;

//use sdl2::event::Event;

pub struct ImguiGLFW {
    last_frame: Instant,
    mouse_press: [bool; 5],
    cursor_pos: (f64, f64),
    ignore_mouse: bool,
    ignore_keyboard: bool,
    cursor: (ImGuiMouseCursor, Option<StandardCursor>),
}

impl ImguiGLFW {
    pub fn new(imgui: &mut ImGui) -> Self {
        // TODO: upstream to imgui-rs
        {
            let io = unsafe { &mut *imgui_sys::igGetIO() };

            //io.get_clipboard_text_fn = Some(glfw::ffi::glfwGetClipboardString);
            //io.set_clipboard_text_fn = Some(set_clipboard_string);
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
            ignore_keyboard: false,
            ignore_mouse: false,
            cursor: (ImGuiMouseCursor::None, None),
        }
    }

    pub fn ignore_event(&self, event: &WindowEvent) -> bool {
        match *event {
            WindowEvent::Key(_, _, _, _) => self.ignore_keyboard,
            WindowEvent::MouseButton(_, _, _)
            | WindowEvent::CursorPos(_, _)
            | WindowEvent::CursorEnter(_)
            | WindowEvent::Scroll(_, _) => self.ignore_mouse,
            _ => false,
        }
    }

    pub fn handle_event(&mut self, imgui: &mut ImGui, event: &WindowEvent) {
        fn set_mod(imgui: &mut ImGui, modifier: Modifiers) {
            let ctrl = modifier.intersects(Modifiers::Control);
            let alt = modifier.intersects(Modifiers::Alt);
            let shift = modifier.intersects(Modifiers::Shift);
            let super_ = modifier.intersects(Modifiers::Super);

            imgui.set_key_ctrl(ctrl);
            imgui.set_key_alt(alt);
            imgui.set_key_shift(shift);
            imgui.set_key_super(super_);
        }

        match *event {
            WindowEvent::Scroll(y, _) => {
                imgui.set_mouse_wheel(y as f32);
            }
            WindowEvent::MouseButton(mouse_btn, action, modifiers) => {
                let index = match mouse_btn {
                    MouseButton::Button1 => 0,
                    MouseButton::Button2 => 1,
                    MouseButton::Button3 => 2,
                    MouseButton::Button4 => 3,
                    MouseButton::Button5 => 4,
                    _ => 5,
                };
                self.mouse_press[index] = true;
            }
            /*Event::TextInput { ref text, .. } => {
                for chr in text.chars() {
                    imgui.add_input_character(chr);
                }
            }*/
            WindowEvent::Key(_, scancode, action, modifier) => {
                set_mod(imgui, modifier);
                match action {
                    Action::Press => {
                        imgui.set_key(scancode as u8, true);
                    }
                    Action::Repeat => {
                        imgui.set_key(scancode as u8, true);
                    }
                    Action::Release => {
                        imgui.set_key(scancode as u8, false);
                    }
                }
            }
            WindowEvent::CursorPos(x, y) => {
                self.cursor_pos = (x, y);
            }
            _ => {}
        }
    }

    pub fn frame<'a>(
        &mut self,
        window: &mut Window,
        imgui: &'a mut ImGui,
        messages: &FlushedMessages<'a, WindowEvent>,
    ) -> imgui::Ui<'a> {
        /*
        let mouse_util = window.get_mouse_button();

        // Merging the mousedown events we received into the current state prevents us from missing
        // clicks that happen faster than a frame
        let mouse_state = messages.mouse_state();
        let mouse_down = [
            self.mouse_press[0] || mouse_state.left(),
            self.mouse_press[1] || mouse_state.right(),
            self.mouse_press[2] || mouse_state.middle(),
            self.mouse_press[3] || mouse_state.x1(),
            self.mouse_press[4] || mouse_state.x2(),
        ];
        imgui.set_mouse_down(mouse_down);*/
        imgui.set_mouse_down(self.mouse_press);
        self.mouse_press = [false; 5];

        //let any_mouse_down = mouse_down.iter().any(|&b| b);
        //mouse_util.capture(any_mouse_down);

        imgui.set_mouse_pos(self.cursor_pos.0 as f32, self.cursor_pos.1 as f32);

        let mouse_cursor = imgui.mouse_cursor();
        if imgui.mouse_draw_cursor() || mouse_cursor == ImGuiMouseCursor::None {
            self.cursor = (ImGuiMouseCursor::None, None);
            //mouse_util.show_cursor(false);
        } else {
            //mouse_util.show_cursor(true);

            if mouse_cursor != self.cursor.0 {
                let glfw_cursor = match mouse_cursor {
                    ImGuiMouseCursor::None => unreachable!("mouse_cursor was None!"),
                    ImGuiMouseCursor::Arrow => StandardCursor::Arrow,
                    ImGuiMouseCursor::TextInput => StandardCursor::IBeam,
                    ImGuiMouseCursor::Move => StandardCursor::Crosshair,
                    ImGuiMouseCursor::ResizeNS => StandardCursor::VResize,
                    ImGuiMouseCursor::ResizeEW => StandardCursor::HResize,
                    ImGuiMouseCursor::ResizeNESW => StandardCursor::Crosshair,
                    ImGuiMouseCursor::ResizeNWSE => StandardCursor::Crosshair,
                };

                window.set_cursor(Some(Cursor::standard(glfw_cursor)));

                self.cursor = (mouse_cursor, Some(glfw_cursor));
            }
        }

        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        let window_size = window.get_size();
        let display_size = window.get_size();

        let frame_size = imgui::FrameSize {
            logical_size: (window_size.0 as f64, window_size.1 as f64),
            hidpi_factor: (display_size.0 as f64) / (window_size.0 as f64),
        };
        let ui = imgui.frame(frame_size, delta_s);

        self.ignore_keyboard = ui.want_capture_keyboard();
        self.ignore_mouse = ui.want_capture_mouse();

        ui
    }
}

/*
#[doc(hidden)]
pub extern "C" fn get_clipboard_string(window: &Window, _user_data: *mut c_void) -> *const c_char {
    window.get_clipboard_string()
}

#[doc(hidden)]
#[cfg_attr(feature = "cargo-clippy", allow(not_unsafe_ptr_arg_deref))]
pub extern "C" fn set_clipboard_string(window: &Window,_user_data: *mut c_void, text: *const c_char) {
    window.set_clipboard_string(String::from(text));
}
*/
