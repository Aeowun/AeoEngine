#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod character;
mod character_custom;
mod editor;
mod engine;
mod project;
mod renderer;
mod scripting;
mod world;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::ContextAttributesBuilder;
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;

use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::Arc;

use winit::event::{DeviceEvent, Event, WindowEvent};

use winit::event_loop::EventLoop;
use winit::raw_window_handle::HasWindowHandle;

use winit::window::{CursorGrabMode, Fullscreen, WindowAttributes};

use engine::app::App;

fn main() {
    #[cfg(not(debug_assertions))]
    {
        let exe_dir = std::env::current_exe()
            .expect("Failed to determine executable path")
            .parent()
            .expect("Executable has no parent directory")
            .to_path_buf();

        std::env::set_current_dir(&exe_dir)
            .expect("Failed to set working directory to executable directory");
    }

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window_attributes = WindowAttributes::default()
        .with_title(concat!("AeoEngine V", env!("CARGO_PKG_VERSION")))
        .with_fullscreen(Some(Fullscreen::Borderless(None)));

    let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

    let (window, gl_config) = display_builder
        .build(&event_loop, ConfigTemplateBuilder::new(), |configs| {
            configs
                .max_by_key(|config| config.num_samples())
                .expect("No OpenGL configurations available")
        })
        .expect("Failed to create OpenGL display");

    let window = window.expect("Failed to create window");

    let raw_window_handle = window
        .window_handle()
        .expect("Failed to get window handle")
        .as_raw();

    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

    let not_current_context = unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attributes)
            .expect("Failed to create OpenGL context")
    };

    let size = window.inner_size();

    let width = NonZeroU32::new(size.width.max(1)).unwrap();

    let height = NonZeroU32::new(size.height.max(1)).unwrap();

    let surface_attributes =
        SurfaceAttributesBuilder::<WindowSurface>::new().build(raw_window_handle, width, height);

    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &surface_attributes)
            .expect("Failed to create window surface")
    };

    let context = not_current_context
        .make_current(&surface)
        .expect("Failed to make OpenGL context current");

    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();

        gl_config.display().get_proc_address(&symbol).cast()
    });

    let mut app = App::new(size.width as f32, size.height as f32);

    let glow_context = unsafe {
        Arc::new(glow::Context::from_loader_function(|symbol| {
            let symbol = CString::new(symbol).unwrap();

            gl_config.display().get_proc_address(&symbol).cast()
        }))
    };

    let mut egui_glow = egui_glow::Painter::new(glow_context.clone(), "", None, false)
        .expect("failed to create egui painter");

    let mut egui_state = egui_winit::State::new(
        egui::Context::default(),
        egui::viewport::ViewportId::ROOT,
        &window,
        Some(window.scale_factor() as f32),
        None,
        None,
    );

    let mut last_applied_mouse: Option<(bool, bool)> = None;

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => {
                let response = egui_state.on_window_event(&window, &event);

                match &event {
                    WindowEvent::CloseRequested => {
                        app.save_project();
                        elwt.exit();
                    }

                    WindowEvent::Resized(size) => {
                        let width = NonZeroU32::new(size.width.max(1)).unwrap();

                        let height = NonZeroU32::new(size.height.max(1)).unwrap();

                        surface.resize(&context, width, height);
                    }

                    _ => {}
                }

                if !response.consumed {
                    app.on_window_event(&event, egui_state.egui_ctx(), &window);
                }
            }

            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                app.on_mouse_motion(delta.0, delta.1);
            }

            Event::AboutToWait => {
                app.update(egui_state.egui_ctx());

                let current_mouse = (
                    app.mouse.cursor_visible,
                    app.mouse.screen_locked,
                );

                if last_applied_mouse != Some(current_mouse) {
                    let grab_mode = if app.mouse.screen_locked {
                        CursorGrabMode::Locked
                    } else {
                        CursorGrabMode::None
                    };

                    if let Err(_err) = window.set_cursor_grab(grab_mode) {
                        if app.mouse.screen_locked {
                            let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                        }
                    }

                    window.set_cursor_visible(app.mouse.cursor_visible);

                    last_applied_mouse = Some(current_mouse);
                }

                app.render();

                let input = egui_state.take_egui_input(&window);

                egui_state.egui_ctx().begin_pass(input);

                app.update_ui(egui_state.egui_ctx());
                if app.exit_requested {
                    elwt.exit();
                    return;
                }
                let full_output = egui_state.egui_ctx().end_pass();
                egui_state.handle_platform_output(&window, full_output.platform_output);

                let paint_jobs = egui_state
                    .egui_ctx()
                    .tessellate(full_output.shapes, full_output.pixels_per_point);

                unsafe {
                    for (id, image_delta) in &full_output.textures_delta.set {
                        egui_glow.set_texture(*id, image_delta);
                    }

                    let size = window.inner_size();

                    egui_glow.paint_primitives(
                        [size.width, size.height],
                        full_output.pixels_per_point,
                        &paint_jobs,
                    );

                    for id in &full_output.textures_delta.free {
                        egui_glow.free_texture(*id);
                    }
                }

                surface
                    .swap_buffers(&context)
                    .expect("Failed to swap buffers");

                window.request_redraw();
            }

            _ => {}
        })
        .expect("Event loop failed");
}
