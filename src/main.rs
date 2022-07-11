//! PBR experiments
//!
//! `main` function is the entry-point
use std::{thread, time::Duration};

use app::AppState;
use camera::{Camera, CameraTyp, Flycam, Orbitalcam};
use eyre::Result;
use glam::Vec3;
use gui::GuiCtx;
use renderer::Renderer;
use scene::Scene;
use sdl2::{keyboard::Scancode, EventPump};

use window::AppWindow;

mod app;

/// A module for working with a basic free camera.
mod camera;

/// All of the code for drawing the GUI using egui.
mod gui;

/// Represents a single gltf 2.0 model (used models only have 1 scene).
mod model;

/// Handles rendering the whole scene.
mod renderer;

/// Abstractions for working with OpenGL.
mod ogl;

/// Handles window creation and egui boilerplate.
mod window;

mod scene;

mod util;

/// Creates the window, configures OpenGL, sets up the scene and begins the render loop.
fn main() -> Result<()> {
    let mut window = AppWindow::new("PBR experiments - Tomáš Král")?;

    ogl::init_debug();

    let mut appstate = AppState::new(&window);
    let mut scene = Scene::init()?;
    let mut renderer = Renderer::new()?;
    let mut flycam = Flycam::new(
        Vec3::new(0.2, 3., 7.5),
        0.05,
        0.05,
        window.width,
        window.height,
    );

    let mut orbitalcam = Orbitalcam::new(2., 0.05, window.width, window.height);
    let mut cam_typ = CameraTyp::Orbital;

    'render_loop: loop {
        window.begin_frame();

        let active_cam: &mut dyn Camera = match cam_typ {
            CameraTyp::Flycam => &mut flycam,
            CameraTyp::Orbital => &mut orbitalcam,
        };

        handle_inputs(&mut window.event_pump, active_cam);

        if let Some(model) = appstate.selected_model {
            renderer.render(&mut scene.models[model], active_cam, &appstate);
        }

        let mut gui_ctx = GuiCtx {
            models: &mut scene.models,
            camera: active_cam,
            cam_typ: &mut cam_typ,
            renderer: &mut renderer,
        };

        appstate.create_gui(&mut gui_ctx, &mut window.egui_ctx);

        let should_quit = window.end_frame();
        if should_quit {
            break 'render_loop;
        }

        thread::sleep(Duration::from_millis(1));
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
