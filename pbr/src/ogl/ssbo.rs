use bytemuck::{Pod, Zeroable};

/// Abstraction for working with SSBOs.
pub struct Ssbo<const BINDING: u32> {
    pub id: u32,
}

impl<const BINDING: u32> Ssbo<BINDING> {
    /// Generate a new SSBO and allocate memory for it
    pub fn new<T: Pod + Zeroable>(buf: &[T]) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);

            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, BINDING, id);

            let buf: &[u8] = bytemuck::cast_slice(buf);
            let buf_len = buf.len() as u32;

            // Buffer initialization
            gl::NamedBufferData(id, buf_len as _, buf.as_ptr() as _, gl::STATIC_COPY);

            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }

        Self { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.id);
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, BINDING, self.id);
        }
    }
}

impl<const BINDING: u32> Drop for Ssbo<BINDING> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}
