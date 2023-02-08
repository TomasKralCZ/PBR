//! PBR experiments
//!
//! `main` function is the entry-point
use std::{thread, time::Duration};

use app_settings::AppSettings;
use camera::{Camera, CameraTyp, Flycam, Orbitalcam};
use eyre::Result;
use glam::Vec3;
use gui::Gui;
use renderer::{RenderCtx, Renderer};
use resources::Resources;
use sdl2::{keyboard::Scancode, EventPump};

use util::RcMut;
use window::AppWindow;

mod app_settings;

/// A module for working with a basic free camera.
mod camera;

/// All of the code for drawing the GUI using egui.
mod gui;

/// Represents a single gltf 2.0 scene (used files only have 1 scene).
mod scene;

/// Handles rendering the whole scene.
mod renderer;

/// Abstractions for working with OpenGL.
mod ogl;

/// Handles window creation and egui boilerplate.
mod window;

mod resources;

mod util;

mod brdf_raw;

/// Creates the window, configures OpenGL, sets up the scene and begins the render loop.
fn main() -> Result<()> {
    let mut window = AppWindow::new("Physically Based Rendering - Tomáš Král")?;

    gl::load_with(|name| window.window.subsystem().gl_get_proc_address(name) as _);
    ogl::init_debug();

    let app_settings = RcMut::new(AppSettings::new(&window));
    let resources = RcMut::new(Resources::init()?);
    let mut renderer = Renderer::new()?;

    let mut gui_ctx = Gui {
        resources: resources.clone(),
        app_settings: app_settings.clone(),
    };

    let mut flycam = Flycam::new(
        Vec3::new(0.2, 3., 7.5),
        0.05,
        0.05,
        window.width,
        window.height,
    );

    let mut orbitalcam = Orbitalcam::new(2., 0.05, window.width, window.height);

    'render_loop: loop {
        window.begin_frame();

        let active_cam: &mut dyn Camera = match app_settings.get().camera_typ {
            CameraTyp::Flycam => &mut flycam,
            CameraTyp::Orbital => &mut orbitalcam,
        };

        handle_inputs(&mut window.event_pump, active_cam);

        {
            let mut rctx = RenderCtx {
                app_settings: &mut app_settings.get_mut(),
                camera: active_cam,
                res: &mut resources.get_mut(),
            };

            renderer.render(&mut rctx)?;
        }

        gui_ctx.create_gui(&mut window.egui_ctx);

        let should_quit = window.end_frame();
        if should_quit {
            break 'render_loop;
        }

        // TODO: precise sleeping
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

/// Modifies camera state based on the mouse / keyboard inputs
fn handle_inputs(event_pump: &mut EventPump, camera: &mut dyn Camera) {
    let k = event_pump.keyboard_state();

    if k.is_scancode_pressed(Scancode::W) {
        camera.on_forward();
    }

    if k.is_scancode_pressed(Scancode::S) {
        camera.on_backward();
    }

    if k.is_scancode_pressed(Scancode::A) {
        camera.on_left();
    }

    if k.is_scancode_pressed(Scancode::D) {
        camera.on_right();
    }

    let mouse_state = event_pump.mouse_state();
    let mouse_x = mouse_state.x() as f32;
    let mouse_y = mouse_state.y() as f32;

    if mouse_state.right() {
        camera.adjust_look(mouse_x, mouse_y);
    } else {
        camera.track_mouse_pos(mouse_x, mouse_y)
    }
}
