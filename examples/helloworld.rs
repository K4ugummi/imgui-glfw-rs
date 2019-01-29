use imgui_glfw_rs::glfw;
use imgui_glfw_rs::imgui;

use glfw::Context;
use imgui::{FontGlyphRange, ImFontConfig, ImGui};
use imgui_glfw_rs::ImguiGLFW;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));

    let (mut window, events) = glfw
        .create_window(
            1024,
            768,
            "imgui-glfw-rs example",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create window");

    window.make_current();
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.set_mouse_button_polling(true);
    window.set_char_polling(true);
    window.set_key_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }

    let mut imgui = ImGui::init();

    imgui.fonts().add_default_font_with_config(
        ImFontConfig::new()
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(24.),
    );

    imgui.fonts().add_font_with_config(
        include_bytes!("../res/OpenSans-Regular.ttf"),
        ImFontConfig::new()
            .merge_mode(true)
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(24.)
            .rasterizer_multiply(1.75),
        &FontGlyphRange::japanese(),
    );

    imgui.set_font_global_scale(1.);

    let mut imgui_glfw = ImguiGLFW::new(&mut imgui);

    let renderer =
        imgui_opengl_renderer::Renderer::new(&mut imgui, |s| window.get_proc_address(s) as _);

    while !window.should_close() {
        window.make_current();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let ui = imgui_glfw.frame(&mut window, &mut imgui);

        ui.show_demo_window(&mut true);

        renderer.render(ui);

        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            imgui_glfw.handle_event(&mut imgui, &event);
        }
    }
}
