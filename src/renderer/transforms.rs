use glam::Mat4;

use crate::ogl::uniform_buffer::UniformBufferElement;

/// Uniform buffer element that stores the transformation matrices
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
#[repr(C)]
pub struct Transforms {
    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
}

impl Transforms {
    pub fn new_indentity() -> Self {
        Self {
            projection: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            model: Mat4::IDENTITY,
        }
    }
}

impl UniformBufferElement for Transforms {
    const BINDING: u32 = 1;
}
