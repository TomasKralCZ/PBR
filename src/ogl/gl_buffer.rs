use super::BufferId;

pub struct GlBuffer {
    pub id: BufferId,
}

impl GlBuffer {
    pub fn new<T: bytemuck::Pod + bytemuck::Zeroable>(buf: &[T]) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::CreateBuffers(1, &mut id);

            let bytes = bytemuck::cast_slice::<T, u8>(buf);

            gl::NamedBufferStorage(id, bytes.len() as isize, bytes.as_ptr() as _, 0);
        }

        Self { id }
    }
}

impl Drop for GlBuffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.id) }
    }
}
