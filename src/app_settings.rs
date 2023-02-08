use crate::{
    camera::CameraTyp,
    renderer::{pbr_settings::PbrSettings, PbrMaterial},
    window::AppWindow,
};

pub struct AppSettings {
    // Index into resources scene vector
    pub selected_scene: usize,
    // Index into resources envmaps vector
    pub selected_envmap: usize,

    pub camera_typ: CameraTyp,

    pub viewport_dim: ViewportDim,

    pub material_src: MaterialSrc,
    pub pbr_material_override: PbrMaterial,
    pub pbr_settings: PbrSettings,
    // Index into the resources merl_brdfs vector
    pub selected_merl_brdf: usize,
    // Index into the resources mit_brdfs vector
    pub selected_mit_brdf: usize,
    // Index into the resources utia_brdfs vector
    pub selected_utia_brdf: usize,
}

impl AppSettings {
    pub fn new(window: &AppWindow) -> Self {
        Self {
            selected_scene: 0,
            selected_envmap: 0,
            viewport_dim: ViewportDim::new(window),
            material_src: MaterialSrc::Gltf,
            camera_typ: CameraTyp::Orbital,
            pbr_material_override: PbrMaterial::new(),
            pbr_settings: PbrSettings::new(),
            selected_merl_brdf: 0,
            selected_mit_brdf: 0,
            selected_utia_brdf: 0,
        }
    }
}

#[repr(u32)]
#[derive(PartialEq, Clone, Copy, bytemuck::NoUninit)]
pub enum DiffuseType {
    Lambert = 0,
    Frostbite = 1,
    CodWWII = 2,
}

impl DiffuseType {
    pub fn to_str(self) -> &'static str {
        match self {
            DiffuseType::Lambert => "Lambert",
            DiffuseType::Frostbite => "Frostbite",
            DiffuseType::CodWWII => "CoD: WWII",
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum MaterialSrc {
    Gltf,
    PbrOverride,
    MerlBrdf,
    MitBrdf,
    UtiaBrdf,
}

impl MaterialSrc {
    pub fn to_str(self) -> &'static str {
        match self {
            MaterialSrc::Gltf => "GLTF",
            MaterialSrc::PbrOverride => "Override material",
            MaterialSrc::MerlBrdf => "MERL BRDF database",
            MaterialSrc::MitBrdf => "MIT CSAIL database",
            MaterialSrc::UtiaBrdf => "UTIA BRDF database",
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
