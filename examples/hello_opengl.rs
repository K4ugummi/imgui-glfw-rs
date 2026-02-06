use glfw::Context;
use imgui::Context as ImContext;
use imgui_glfw_rs::ImguiGLFW;
use imgui_glfw_rs::glfw;

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));

    let (mut window, events) = glfw
        .create_window(
            1024,
            768,
            "imgui-glfw-rs OpenGL example",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create window");

    window.make_current();
    window.set_all_polling(true);

    gl::load_with(|symbol| {
        window
            .get_proc_address(symbol)
            .map_or(std::ptr::null(), |f| f as *const _)
    });
    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }

    let mut imgui = ImContext::create();

    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let mut imgui_glfw =
        ImguiGLFW::new(&mut imgui, &mut window).expect("Failed to create renderer");

    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let ui = imgui_glfw.frame(&mut window, &mut imgui);

        // Render imgui's demo window
        ui.show_demo_window(&mut true);

        // Render a custom imgui window
        ui.window("Hello Window")
            .size([300., 300.], imgui::Condition::Once)
            .collapsible(false)
            .build(|| {
                ui.text("This is some random text");
                if ui.button("Open Modal") {
                    ui.open_popup("Modal Popup");
                }
            });

        ui.modal_popup_config("Modal Popup").build(|| {
            ui.text("This is a modal popup.");
            if ui.button_with_size("Close", [200., 40.]) {
                ui.close_current_popup();
            }
        });

        imgui_glfw.draw(&mut imgui, &mut window);

        window.swap_buffers();

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            let captured = imgui_glfw.handle_event(&mut imgui, &event);
            if captured {
                // imgui wants this event; skip forwarding to app logic
            }
        }
    }
}
