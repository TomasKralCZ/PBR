use std::time::Instant;

use egui::CtxRef;
use egui_backend::{painter::Painter, DpiScaling, EguiStateHandler};
use egui_sdl2_gl::ShaderVersion;
use eyre::{eyre, Result};
use sdl2::{
    event::{Event, WindowEvent},
    video::Window,
    video::{GLContext, GLProfile, SwapInterval},
    EventPump, Sdl, VideoSubsystem,
};

use egui_sdl2_gl as egui_backend;

/// A component that handles the window creation and egui drawing
pub struct AppWindow {
    /// SDL2 context
    _sdl_context: Sdl,
    /// SDL2 video subsystem
    _video_subsystem: VideoSubsystem,
    /// SDL2 window
    pub window: Window,
    /// The OpenGL context
    _gl_ctx: GLContext,
    /// SDL2 event pump
    pub event_pump: EventPump,
    /// The egui context
    pub egui_ctx: CtxRef,
    /// Egui state
    egui_state: EguiStateHandler,
    /// egui_sdl2_gl rednerer
    painter: Painter,
    /// Time when the window was creaetd
    start_time: Instant,
    /// Width of the window
    pub width: u32,
    /// Height of the window
    pub height: u32,
}

impl AppWindow {
    /// Create the window with the specific title.
    ///
    /// Initializes the OpenGL context, the egui renderer and the SDL2 video subsystem.
    pub fn new(title: &str) -> Result<Self> {
        let sdl_context = sdl2::init().map_err(|e| eyre!("{e}"))?;
        let video_subsystem = sdl_context.video().map_err(|e| eyre!("{e}"))?;

        let size = video_subsystem
            .display_bounds(0)
            .map_err(|e| eyre!("{e}"))?;

        let width = (size.width() as f32 * 0.7) as u32;
        let height = (size.height() as f32 * 0.7) as u32;

        // Init OpenGL
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_major_version(4);
        gl_attr.set_context_minor_version(6);
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_flags().debug().set();
        gl_attr.set_double_buffer(true);

        let window = video_subsystem
            .window(title, width, height)
            .opengl()
            .resizable()
            .position_centered()
            .allow_highdpi()
            .build()?;

        let gl_ctx = window.gl_create_context().map_err(|e| eyre!("{e}"))?;

        window
            .subsystem()
            .gl_set_swap_interval(SwapInterval::Immediate)
            .map_err(|e| eyre!("{e}"))?;

        let shader_ver = ShaderVersion::Default;

        // It's better if we calculate this ourselves
        let custom_dpi = {
            if width <= 1280 && height <= 720 {
                1.0
            } else if width <= 1920 && height <= 1080 {
                1.5
            } else {
                2.5
            }
        };

        let (painter, egui_state) =
            egui_backend::with_sdl2(&window, shader_ver, DpiScaling::Custom(custom_dpi));
        let egui_ctx = egui::CtxRef::default();
        let event_pump = sdl_context.event_pump().map_err(|e| eyre!("{e}"))?;

        Ok(Self {
            _sdl_context: sdl_context,
            _video_subsystem: video_subsystem,
            window,
            _gl_ctx: gl_ctx,
            event_pump,
            egui_ctx,
            egui_state,
            painter,
            start_time: Instant::now(),
            width,
            height,
        })
    }

    /// Resets state at the beginning of a frame
    pub fn begin_frame(&mut self) {
        self.egui_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
        self.egui_ctx.begin_frame(self.egui_state.input.take());
    }

    /// Finalizes the frame and returns if the render loop should terminate
    pub fn end_frame(&mut self) -> bool {
        unsafe {
            // Reset gl properties so Egui can render properly
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }

        let (egui_output, paint_cmds) = self.egui_ctx.end_frame();
        // Process ouput
        self.egui_state.process_output(&self.window, &egui_output);

        let paint_jobs = self.egui_ctx.tessellate(paint_cmds);

        if !egui_output.needs_repaint {
            // TODO: check egui_backend needs_repaint
            /* if let Some(event) = self.event_pump.wait_event_timeout(5) {
                match event {
                    Event::Quit { .. } => return true,
                    _ => {
                        self.egui_state
                            .process_input(&self.window, event, &mut self.painter);
                    }
                }
            } */
        } else {
            self.painter
                .paint_jobs(None, paint_jobs, &self.egui_ctx.font_image());
            self.window.gl_swap_window();
        }
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return true,
                Event::Window {
                    timestamp: _,
                    window_id: _,
                    win_event: WindowEvent::Resized(new_width, new_height),
                } => {
                    self.width = new_width as u32;
                    self.height = new_height as u32;
                }
                _ => {
                    self.egui_state
                        .process_input(&self.window, event, &mut self.painter);
                }
            }
        }

        false
    }
}
