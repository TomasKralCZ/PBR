use std::mem::size_of;

use shader_constants::CONSTS;

use crate::ogl::{gl_buffer::GlBuffer, vao::Vao};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pos: [f32; 3],
}

#[rustfmt::skip]
pub const VERTICES: [Vertex; 8] = [
    Vertex{pos: [-1., -1., 1.]},
    Vertex{pos: [1., -1., 1.]},
    Vertex{pos: [1., 1., 1.]},
    Vertex{pos: [-1., 1., 1.]},
    Vertex{pos: [-1., -1., -1.]},
    Vertex{pos: [1., -1., -1.]},
    Vertex{pos: [1., 1., -1.]},
    Vertex{pos: [-1., 1., -1.]},
];

#[rustfmt::skip]
pub const INDICES: [u8; 36] = [
    0, 2, 1,
    0, 3, 2,
    1, 6, 5,
    1, 2, 6,
    5, 7, 4,
    5, 6, 7,
    4, 3, 0,
    4, 7, 3,
    3, 7, 6,
    3, 6, 2,
    4, 0, 1,
    4, 1, 5,
];

pub fn init_cube() -> Vao {
    let vao = Vao::new();

    let vertex_buf = GlBuffer::new(&VERTICES);
    vao.attach_vertex_buf(
        &vertex_buf,
        3,
        CONSTS.vertex_attrib_indices.position,
        gl::FLOAT,
        size_of::<Vertex>(),
    );

    let index_buf = GlBuffer::new(&INDICES);
    vao.attach_index_buffer(&index_buf);

    vao
}
