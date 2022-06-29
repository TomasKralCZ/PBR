//! PBR experiments
//!
//! `main` function is the entry-point
use std::{thread, time::Duration};

use camera::Camera;
use eyre::Result;
use glam::Vec3;
use gui::{Ctx, Gui};
use renderer::Renderer;
use scene::Scene;
use sdl2::{keyboard::Scancode, EventPump};

use window::MyWindow;

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

/// Creates the window, configures OpenGL, sets up the scene and begins the render loop.
fn main() -> Result<()> {
    let mut window = MyWindow::new("PBR experiments - Tomáš Král")?;

    ogl::init_debug();

    let mut scene = Scene::init()?;
    let mut gui = Gui::new();
    let mut renderer = Renderer::new()?;
    let mut camera = Camera::new(
        Vec3::new(0.2, 3., 7.5),
        0.05,
        0.05,
        window.width,
        window.height,
    );

    'render_loop: loop {
        handle_inputs(&mut window.event_pump, &mut camera);

        window.begin_frame();

        renderer.render(&mut scene, &mut camera, &window, &gui);

        let gui_ctx = Ctx {
            models: &mut scene.models,
            camera: &mut camera,
            renderer: &mut renderer,
        };

        gui.create_gui(gui_ctx, &mut window.egui_ctx);

        let should_quit = window.end_frame();
        if should_quit {
            break 'render_loop;
        }

        thread::sleep(Duration::from_millis(3));
    }

    Ok(())
}

/// Modifies camera state based on the mouse / keyboard inputs
fn handle_inputs(event_pump: &mut EventPump, camera: &mut Camera) {
    let k = event_pump.keyboard_state();

    if k.is_scancode_pressed(Scancode::W) {
        camera.move_forward(1.0);
    }

    if k.is_scancode_pressed(Scancode::S) {
        camera.move_backward(1.0);
    }

    if k.is_scancode_pressed(Scancode::A) {
        camera.strafe_left(1.0);
    }

    if k.is_scancode_pressed(Scancode::D) {
        camera.strafe_right(1.0);
    }

    let mouse_state = event_pump.mouse_state();
    let mouse_x = mouse_state.x() as f32;
    let mouse_y = mouse_state.y() as f32;

    if mouse_state.right() {
        camera.adjust_look(mouse_x, mouse_y);
    } else {
        camera.set_x_y(mouse_x, mouse_y)
    }
}
