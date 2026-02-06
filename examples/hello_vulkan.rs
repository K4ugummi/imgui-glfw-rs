use std::ffi::CString;

use ash::vk::{self, Handle};
use imgui::Context as ImContext;
use imgui_glfw_rs::ImguiGLFW;
use imgui_glfw_rs::glfw;
use imgui_glfw_rs::imgui_vulkan_renderer_rs::{Renderer, RendererCreateInfo};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const MAX_FRAMES_IN_FLIGHT: usize = 2;

fn main() {
    // --- GLFW init (no OpenGL context) ---
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

    let (mut window, events) = glfw
        .create_window(
            WIDTH,
            HEIGHT,
            "imgui-glfw-rs Vulkan example",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create window");

    window.set_all_polling(true);

    // --- Vulkan instance ---
    let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan") };

    let required_extensions = glfw
        .get_required_instance_extensions()
        .expect("GLFW: Vulkan not supported");
    let extension_ptrs: Vec<CString> = required_extensions
        .iter()
        .map(|s| CString::new(s.as_str()).unwrap())
        .collect();
    let extension_raw: Vec<*const i8> = extension_ptrs.iter().map(|s| s.as_ptr()).collect();

    let app_info = vk::ApplicationInfo::default()
        .application_name(c"imgui-glfw-rs Vulkan example")
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(c"No Engine")
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(vk::API_VERSION_1_0);

    let instance_ci = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(&extension_raw);

    let instance = unsafe {
        entry
            .create_instance(&instance_ci, None)
            .expect("Failed to create Vulkan instance")
    };

    // --- Surface ---
    let surface = {
        let mut surface_raw = vk::SurfaceKHR::null();
        let result = unsafe {
            window.create_window_surface(
                instance.handle().as_raw() as _,
                std::ptr::null(),
                &mut surface_raw as *mut vk::SurfaceKHR as *mut _,
            )
        };
        assert_eq!(
            result,
            vk::Result::SUCCESS.as_raw(),
            "Failed to create surface"
        );
        surface_raw
    };

    let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

    // --- Physical device ---
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };
    assert!(
        !physical_devices.is_empty(),
        "No Vulkan physical devices found"
    );

    let (physical_device, queue_family_index) = physical_devices
        .iter()
        .find_map(|&pdev| {
            let queue_families =
                unsafe { instance.get_physical_device_queue_family_properties(pdev) };
            queue_families.iter().enumerate().find_map(|(i, qf)| {
                let supports_graphics = qf.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                let supports_present = unsafe {
                    surface_loader
                        .get_physical_device_surface_support(pdev, i as u32, surface)
                        .unwrap_or(false)
                };
                if supports_graphics && supports_present {
                    Some((pdev, i as u32))
                } else {
                    None
                }
            })
        })
        .expect("No suitable physical device found");

    let memory_properties =
        unsafe { instance.get_physical_device_memory_properties(physical_device) };

    // --- Logical device ---
    let queue_priorities = [1.0_f32];
    let queue_ci = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family_index)
        .queue_priorities(&queue_priorities);

    let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
    let device_ci = vk::DeviceCreateInfo::default()
        .queue_create_infos(std::slice::from_ref(&queue_ci))
        .enabled_extension_names(&device_extensions);

    let device = unsafe {
        instance
            .create_device(physical_device, &device_ci, None)
            .expect("Failed to create logical device")
    };
    let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

    // --- Swapchain ---
    let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

    let surface_caps = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface)
            .unwrap()
    };
    let surface_formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(physical_device, surface)
            .unwrap()
    };

    let surface_format = surface_formats
        .iter()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_UNORM
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or(&surface_formats[0]);

    let image_count = {
        let desired = surface_caps.min_image_count + 1;
        if surface_caps.max_image_count > 0 {
            desired.min(surface_caps.max_image_count)
        } else {
            desired
        }
    };

    let extent = if surface_caps.current_extent.width != u32::MAX {
        surface_caps.current_extent
    } else {
        vk::Extent2D {
            width: WIDTH,
            height: HEIGHT,
        }
    };

    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface)
            .unwrap()
    };
    let present_mode = if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else {
        vk::PresentModeKHR::FIFO
    };

    let swapchain_ci = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(surface_caps.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_ci, None)
            .expect("Failed to create swapchain")
    };

    let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };

    // --- Image views ---
    let image_views: Vec<vk::ImageView> = swapchain_images
        .iter()
        .map(|&image| {
            let ci = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .level_count(1)
                        .layer_count(1),
                );
            unsafe { device.create_image_view(&ci, None).unwrap() }
        })
        .collect();

    // --- Render pass ---
    let attachment = vk::AttachmentDescription::default()
        .format(surface_format.format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_ref));

    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let render_pass_ci = vk::RenderPassCreateInfo::default()
        .attachments(std::slice::from_ref(&attachment))
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(std::slice::from_ref(&dependency));

    let render_pass = unsafe {
        device
            .create_render_pass(&render_pass_ci, None)
            .expect("Failed to create render pass")
    };

    // --- Framebuffers ---
    let framebuffers: Vec<vk::Framebuffer> = image_views
        .iter()
        .map(|iv| {
            let ci = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(std::slice::from_ref(iv))
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            unsafe { device.create_framebuffer(&ci, None).unwrap() }
        })
        .collect();

    // --- Command pool + buffers ---
    let command_pool_ci = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family_index);

    let command_pool = unsafe {
        device
            .create_command_pool(&command_pool_ci, None)
            .expect("Failed to create command pool")
    };

    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32);

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&alloc_info)
            .expect("Failed to allocate command buffers")
    };

    // --- Sync (per frame in flight) ---
    let fence_ci = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
    let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
    let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
    let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        in_flight_fences.push(unsafe { device.create_fence(&fence_ci, None).unwrap() });
        image_available_semaphores.push(unsafe {
            device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .unwrap()
        });
        render_finished_semaphores.push(unsafe {
            device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .unwrap()
        });
    }
    let mut current_frame: usize = 0;

    // --- imgui ---
    let mut imgui = ImContext::create();
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let mut imgui_glfw = ImguiGLFW::new(&mut imgui, &mut window);

    let mut renderer = Renderer::new(
        &mut imgui,
        &RendererCreateInfo {
            device: device.clone(),
            memory_properties,
            render_pass,
            command_pool,
            queue,
        },
    )
    .expect("Failed to create imgui Vulkan renderer");

    // --- Main loop ---
    while !window.should_close() {
        let fence = in_flight_fences[current_frame];
        let image_available = image_available_semaphores[current_frame];
        let render_finished = render_finished_semaphores[current_frame];

        unsafe {
            device
                .wait_for_fences(&[fence], true, u64::MAX)
                .unwrap();
            device.reset_fences(&[fence]).unwrap();
        }

        let (image_index, _suboptimal) = unsafe {
            swapchain_loader
                .acquire_next_image(swapchain, u64::MAX, image_available, vk::Fence::null())
                .expect("Failed to acquire swapchain image")
        };

        let cmd = command_buffers[current_frame];

        // Build imgui frame
        let ui = imgui_glfw.frame(&mut window, &mut imgui);

        ui.show_demo_window(&mut true);

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

        imgui_glfw.update_cursors(&imgui, &mut window);
        let draw_data = imgui.render();

        // Record command buffer
        unsafe {
            device
                .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
                .unwrap();
            device
                .begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default())
                .unwrap();

            let clear_value = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            };
            let rp_begin = vk::RenderPassBeginInfo::default()
                .render_pass(render_pass)
                .framebuffer(framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D::default(),
                    extent,
                })
                .clear_values(std::slice::from_ref(&clear_value));

            device.cmd_begin_render_pass(cmd, &rp_begin, vk::SubpassContents::INLINE);
            renderer
                .render(draw_data, cmd)
                .expect("Failed to render imgui");
            device.cmd_end_render_pass(cmd);
            device.end_command_buffer(cmd).unwrap();
        }

        // Submit
        let wait_semaphores = [image_available];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [render_finished];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(&cmd))
            .signal_semaphores(&signal_semaphores);

        unsafe {
            device
                .queue_submit(queue, &[submit_info], fence)
                .expect("Failed to submit command buffer");
        }

        // Present
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(std::slice::from_ref(&swapchain))
            .image_indices(std::slice::from_ref(&image_index));

        unsafe {
            swapchain_loader
                .queue_present(queue, &present_info)
                .expect("Failed to present");
        }

        current_frame = (current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        // Poll events
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            let captured = imgui_glfw.handle_event(&mut imgui, &event);
            if captured {
                // imgui wants this event; skip forwarding to app logic
            }
        }
    }

    // --- Cleanup ---
    unsafe {
        device.device_wait_idle().unwrap();

        // renderer is dropped automatically

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            device.destroy_semaphore(render_finished_semaphores[i], None);
            device.destroy_semaphore(image_available_semaphores[i], None);
            device.destroy_fence(in_flight_fences[i], None);
        }
        device.destroy_command_pool(command_pool, None);
        for fb in &framebuffers {
            device.destroy_framebuffer(*fb, None);
        }
        device.destroy_render_pass(render_pass, None);
        for iv in &image_views {
            device.destroy_image_view(*iv, None);
        }
        swapchain_loader.destroy_swapchain(swapchain, None);
        device.destroy_device(None);
        surface_loader.destroy_surface(surface, None);
        instance.destroy_instance(None);
    }
}
