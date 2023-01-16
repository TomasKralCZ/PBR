use crate::{
    camera::CameraTyp,
    renderer::{pbr_settings::PbrSettings, PbrMaterial},
    window::AppWindow,
};

pub struct AppSettings {
    // Index into resources scene vector
    pub selected_scene: usize,

    pub camera_typ: CameraTyp,

    pub viewport_dim: ViewportDim,

    pub should_override_material: bool,
    pub pbr_material_override: PbrMaterial,

    pub pbr_settings: PbrSettings,

    pub data_driven_rendering: bool,
    // Index into the resources brdf vector
    pub selected_brdf: usize,
}

impl AppSettings {
    pub fn new(window: &AppWindow) -> Self {
        Self {
            selected_scene: 0,
            viewport_dim: ViewportDim::new(window),
            camera_typ: CameraTyp::Orbital,
            should_override_material: false,
            pbr_material_override: PbrMaterial::new(),
            pbr_settings: PbrSettings::new(),
            data_driven_rendering: false,
            selected_brdf: 0,
        }
    }
}

pub struct ViewportDim {
    pub min_x: f32,
    pub min_y: f32,
    pub width: f32,
    pub height: f32,
}

impl ViewportDim {
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
