use std::ffi::CStr;

use gl::types::GLenum;

use super::TextureId;

pub struct GlTexture {
    pub id: TextureId,
}

impl GlTexture {
    pub fn new(typ: GLenum) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::CreateTextures(typ, 1, &mut id);
        }

        Self { id }
    }

    pub fn add_label(&self, label: &CStr) {
        unsafe {
            gl::ObjectLabel(
                gl::TEXTURE,
                self.id,
                label.to_bytes().len() as _,
                label.as_ptr(),
            );
        }
    }
}

impl Drop for GlTexture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id) }
    }
}
