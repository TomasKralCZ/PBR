use crate::{camera::CameraTyp, renderer::PbrMaterial, window::AppWindow};

pub struct AppState {
    pub camera_typ: CameraTyp,
    pub selected_model: Option<usize>,
    pub render_viewport_dim: RenderViewportDim,
    pub should_override_material: bool,
    pub pbr_material_override: PbrMaterial,
}

impl AppState {
    pub fn new(window: &AppWindow) -> Self {
        Self {
            camera_typ: CameraTyp::Orbital,
            selected_model: None,
            render_viewport_dim: RenderViewportDim::new(window),
            should_override_material: false,
            pbr_material_override: PbrMaterial::new(),
        }
    }
}

pub struct RenderViewportDim {
    pub min_x: f32,
    pub min_y: f32,
    pub width: f32,
    pub height: f32,
}

impl RenderViewportDim {
    pub fn new(window: &AppWindow) -> Self {
        let width = window.width as f32;
        let height = window.height as f32;

        Self {
            min_x: 0.,
            min_y: 0.,
            width,
            height,
        }
    }
}
