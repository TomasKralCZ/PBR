use gl::types::GLenum;

use crate::ogl;

#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub texcoords: [f32; 2],
    pub tangent: [f32; 4],
}

impl Vertex {
    pub const ATTRIB_SIZES: [i32; 4] = [3, 3, 2, 4];
    pub const ATTRIB_INDICES: [u32; 4] = [
        ogl::POSITION_INDEX,
        ogl::NORMALS_INDEX,
        ogl::TEXCOORDS_INDEX,
        ogl::TANGENT_INDEX,
    ];

    pub const ATTRIB_TYPES: [GLenum; 4] = [gl::FLOAT; 4];
    pub const ATTRIB_OFFSETS: [usize; 4] = [0, 12, 24, 32];
}
