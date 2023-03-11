use shader_constants::CONSTS;

use crate::{app_settings::DiffuseType, ogl::uniform_buffer::UniformBufferElement};

#[derive(bytemuck::NoUninit, Copy, Clone, PartialEq)]
#[repr(C)]
/// Runtime shader settings
pub struct PbrSettings {
    // bools are 4-byte in GLSL std140...
    clearcoat_enabled: u32,
    direct_light_enabled: u32,
    ibl_enabled: u32,
    pub diffuse_type: DiffuseType,
    energycomp_enabled: u32,
}

impl PbrSettings {
    pub fn new() -> Self {
        Self {
            clearcoat_enabled: 1,
            direct_light_enabled: 1,
            ibl_enabled: 1,
            diffuse_type: DiffuseType::Lambert,
            energycomp_enabled: 1,
        }
    }

    pub fn set_clearcoat_enabled(&mut self, clearcoat_enabled: bool) {
        self.clearcoat_enabled = if clearcoat_enabled { 1 } else { 0 };
    }

    pub fn set_direct_light_enabled(&mut self, direct_light_enabled: bool) {
        self.direct_light_enabled = if direct_light_enabled { 1 } else { 0 };
    }

    pub fn set_ibl_enabled(&mut self, ibl_enabled: bool) {
        self.ibl_enabled = if ibl_enabled { 1 } else { 0 };
    }

    pub fn set_energycomp_enabled(&mut self, energycomp_enabled: bool) {
        self.energycomp_enabled = if energycomp_enabled { 1 } else { 0 };
    }

    pub fn clearcoat_enabled(&self) -> bool {
        self.clearcoat_enabled != 0
    }

    pub fn direct_light_enabled(&self) -> bool {
        self.direct_light_enabled != 0
    }

    pub fn ibl_enabled(&self) -> bool {
        self.ibl_enabled != 0
    }

    pub fn energycomp_enabled(&self) -> bool {
        self.energycomp_enabled != 0
    }
}

impl UniformBufferElement for PbrSettings {
    const BINDING: u32 = CONSTS.buffer_bindings.settings;
}
