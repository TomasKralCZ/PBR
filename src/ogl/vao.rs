use super::{gl_buffer::GlBuffer, VaoId};

pub struct Vao {
    pub id: VaoId,
}

impl Vao {
    pub fn new() -> Self {
        let mut id = 0;

        unsafe {
            gl::CreateVertexArrays(1, &mut id);
        }

        Self { id }
    }

    /// Attach a vertex buffer with single vertex attribute
    pub fn attach_vertex_buf(
        &self,
        buffer: &GlBuffer,
        components: i32,
        attrib_index: u32,
        typ: u32,
        element_size: usize,
    ) {
        unsafe {
            let buf_bind_index = 0;

            gl::VertexArrayVertexBuffer(self.id, buf_bind_index, buffer.id, 0, element_size as i32);

            gl::EnableVertexArrayAttrib(self.id, attrib_index);
            gl::VertexArrayAttribFormat(self.id, attrib_index, components, typ, gl::FALSE, 0 as _);
            gl::VertexArrayAttribBinding(self.id, attrib_index, buf_bind_index);
        }
    }

    /// Attach a vertex buffer with multiple vertex attributes.
    ///
    /// 'components', 'attrib index' and 'typ' have the same meaning as the respective
    /// arguments in glVertexAttribPointer.
    pub fn attach_vertex_buf_multiple_attribs(
        &self,
        buffer: &GlBuffer,
        components: &[i32],
        attrib_indexes: &[u32],
        types: &[u32],
        stride: usize,
        offsets: &[usize],
    ) {
        unsafe {
            let buf_bind_index = 0;

            gl::VertexArrayVertexBuffer(self.id, buf_bind_index, buffer.id, 0, stride as i32);

            for i in 0..attrib_indexes.len() {
                gl::EnableVertexArrayAttrib(self.id, attrib_indexes[i]);
                gl::VertexArrayAttribFormat(
                    self.id,
                    attrib_indexes[i],
                    components[i],
                    types[i],
                    gl::FALSE,
                    offsets[i] as _,
                );
                gl::VertexArrayAttribBinding(self.id, attrib_indexes[i], buf_bind_index);
            }
        }
    }

    pub fn attach_index_buffer(&self, index_buf: &GlBuffer) {
        unsafe {
            gl::VertexArrayElementBuffer(self.id, index_buf.id);
        }
    }
}

impl Drop for Vao {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, &self.id) }
    }
}
