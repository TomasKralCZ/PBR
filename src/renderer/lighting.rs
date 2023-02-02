use glam::Vec4;

use crate::ogl::{self, uniform_buffer::UniformBufferElement};

/// Uniform buffer element that stores the lighing data
#[derive(bytemuck::NoUninit, Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct Lighting {
    pub light_pos: [Vec4; 4],
    pub light_color: [Vec4; 4],
    pub cam_pos: Vec4,
    pub lights: u32,
    padding: [u32; 3],
}

impl Lighting {
    pub fn new() -> Self {
        let cam_pos = Vec4::ZERO;
        let lights = 1;
        let light_pos = [
            Vec4::new(-1., 1., 1., 1.0),
            Vec4::new(10., 10., 10.0, 1.0),
            Vec4::ZERO,
            Vec4::ZERO,
        ];

        let light_color = [
            Vec4::new(1., 1., 1., 0.),
            Vec4::new(1., 1., 1., 0.),
            Vec4::ONE,
            Vec4::ONE,
        ];

        Self {
            cam_pos,
            light_pos,
            light_color,
            lights,
            padding: [0; 3],
        }
    }
}

impl UniformBufferElement for Lighting {
    const BINDING: u32 = ogl::LIGHTNING_BINDING;
}
