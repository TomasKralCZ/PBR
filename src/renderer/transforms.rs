use glam::Mat4;
use shader_constants::CONSTS;

use crate::ogl::uniform_buffer::UniformBufferElement;

/// Uniform buffer element that stores the transformation matrices
#[derive(bytemuck::NoUninit, Copy, Clone, PartialEq, Debug)]
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
    const BINDING: u32 = CONSTS.buffer_bindings.transforms;
}
