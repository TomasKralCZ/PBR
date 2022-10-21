use crate::{camera::CameraTyp, gui::RenderViewportDim, renderer::PbrMaterial, window::AppWindow};

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
