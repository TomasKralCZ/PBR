use std::{mem::size_of, ptr};

use bytemuck::{Pod, Zeroable};

/// Abstraction for working with UniformBuffers.
/// UniformBuffer is generic over T, and T must implement the UniformBufferElement trait.
pub struct UniformBuffer<T: UniformBufferElement> {
    pub id: u32,
    pub inner: T,
}

impl<T: UniformBufferElement> UniformBuffer<T>
where
    T: UniformBufferElement,
{
    /// Generate a new UniformBuffer and allocate memory for it
    pub fn new(inner: T) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::UNIFORM_BUFFER, id);

            let binding = T::BINDING;
            gl::BindBufferBase(gl::UNIFORM_BUFFER, binding, id);

            inner.init_buffer();
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }

        let mut s = Self { id, inner };
        s.update();
        s
    }

    /// Update the UniformBuffer with the current data
    pub fn update(&mut self) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.id);

            self.inner.update();

            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }
}

pub trait UniformBufferElement:
    Sized + Zeroable + Pod + Clone + PartialEq + std::fmt::Debug
{
    /// The binding port
    const BINDING: u32;
    /// Update buffer data using gl::BufferSubData
    fn update(&self) {
        let buf = bytemuck::bytes_of(self);

        unsafe {
            gl::BufferSubData(gl::UNIFORM_BUFFER, 0, buf.len() as isize, buf.as_ptr() as _);
        }
    }

    /// Allocate data for the element with gl::BufferData
    fn init_buffer(&self) {
        unsafe {
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size_of::<Self>() as isize,
                ptr::null() as _,
                gl::DYNAMIC_DRAW,
            );
        }
    }
}

impl<T: UniformBufferElement> Drop for UniformBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}
