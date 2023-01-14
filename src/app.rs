use crate::{camera::CameraTyp, renderer::PbrMaterial};

pub struct AppState {
    pub camera_typ: CameraTyp,
    pub should_override_material: bool,
    pub pbr_material_override: PbrMaterial,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            camera_typ: CameraTyp::Orbital,
            should_override_material: false,
            pbr_material_override: PbrMaterial::new(),
        }
    }
}
