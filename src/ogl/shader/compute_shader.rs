use std::fs;

use eyre::{Context, Result};

use super::{Shader, ShaderId};

pub struct ComputeShader {
    shader_id: ShaderId,
}

impl ComputeShader {
    /// Use this compute shader.
    ///
    /// Draw calls should be passed using the `render` function parameter.
    pub fn _use<F>(&self, compute: F)
    where
        F: FnOnce(),
    {
        unsafe {
            gl::UseProgram(self.shader_id);

            compute();

            gl::UseProgram(0);
        }
    }

    /// Creates a new compute shader from
    pub fn with_path(vs_path: &str) -> Result<ComputeShader> {
        let mut comp_src = String::from_utf8(
            fs::read(vs_path).wrap_err("Couldn't load the compute shader file")?,
        )?;

        // Add null terminator !
        comp_src.push('\0');

        Shader::handle_imports(&mut comp_src)?;
        let comp_shader = Shader::compile_shader(comp_src.as_bytes(), gl::COMPUTE_SHADER)?;
        let comp_shader = Self::create_compute_shader_program(comp_shader)?;

        Ok(Self {
            shader_id: comp_shader,
        })
    }

    fn create_compute_shader_program(comp: ShaderId) -> Result<ShaderId> {
        unsafe {
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, comp);
            gl::LinkProgram(shader_program);

            Shader::check_shader_program_errors(shader_program)?;

            gl::DeleteShader(comp);

            Ok(shader_program)
        }
    }
}
