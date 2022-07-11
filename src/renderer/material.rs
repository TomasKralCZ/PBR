use crate::ogl::uniform_buffer::UniformBufferElement;

/// Uniform buffer element that stores the material settings
#[derive(Default, bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
#[repr(C)]
pub struct Material {
    pub base_color_factor: [f32; 4],
    pub emissive_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
}

impl Material {
    pub fn new() -> Self {
        Self {
            base_color_factor: [1.; 4],
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            normal_scale: 1.0,
            occlusion_strength: 1.0,
            emissive_factor: [0.; 4],
        }
    }
}

impl UniformBufferElement for Material {
    const BINDING: u32 = 4;
}
