use eyre::{eyre, Context, Result};
use gl::types::GLenum;
use glam::{Mat4, Vec3, Vec4};
use std::{ffi::CStr, fs, ptr};

use super::{ProgramId, ShaderId};

pub mod shader_permutations;

/// Represents an OpenGL shader.
///
/// Allows setting uniforms with set_<> methods.
///
/// Use the `draw_with` method for draw calls.
#[derive(Clone)]
pub struct Shader {
    pub program_id: ProgramId,
}

impl Shader {
    /// Use this shader to render.
    ///
    /// Draw calls or compute dispatch should be done in the `render` function parameter.
    pub fn use_shader<F>(&self, render: F)
    where
        F: FnOnce(),
    {
        unsafe {
            gl::UseProgram(self.program_id);

            render();

            gl::UseProgram(0);
        }
    }

    /// Loads a vertex shader and a fragment shader from specified paths and tries to create a shader program.
    pub fn with_files(vs_path: &str, fs_path: &str) -> Result<Shader> {
        let vs_src =
            String::from_utf8(fs::read(vs_path).wrap_err("Couldn't load the vertex shader file")?)?;
        let fs_src = String::from_utf8(
            fs::read(fs_path).wrap_err("Couldn't load the fragment shader file")?,
        )?;

        Self::with_src_defines(vs_src, &[], fs_src, &[])
    }

    /// Tries to compile vertex and fragment shaders from respective paths with respective defines and
    /// create a shader program from them.
    pub fn with_src_defines(
        mut vs_src: String,
        vs_defines: &[&str],
        mut fs_src: String,
        fs_defines: &[&str],
    ) -> Result<Shader> {
        let vs = Self::compile_shader(&mut vs_src, vs_defines, gl::VERTEX_SHADER)?;
        let fs = Self::compile_shader(&mut fs_src, fs_defines, gl::FRAGMENT_SHADER)?;

        let program_id = Self::create_shader_program(&[vs, fs])?;
        Ok(Shader { program_id })
    }

    /// Creates a new compute shader from the path
    pub fn comp_with_path(comp_path: &str) -> Result<Self> {
        let mut comp_src = String::from_utf8(
            fs::read(comp_path).wrap_err("Couldn't load the compute shader file")?,
        )?;

        // Add null terminator !
        comp_src.push('\0');

        let cs = Self::compile_shader(&mut comp_src, &[], gl::COMPUTE_SHADER)?;
        let program_id = Self::create_shader_program(&[cs])?;

        Ok(Self { program_id })
    }

    fn compile_shader(src: &mut String, defines: &[&str], typ: GLenum) -> Result<ShaderId> {
        // Null-terminator
        src.push('\0');

        // Preprocessing steps
        Self::handle_imports(src)?;

        if !defines.is_empty() {
            Self::handle_defines(src, defines)?;
        }

        Self::compile_shader_glsl(src.as_bytes(), typ)
            .wrap_err_with(|| format!("Error compiling shader, defines: '{:?}'", defines))
    }

    /// Tries to compile a shader and checks for compilation errors.
    fn compile_shader_glsl(src: &[u8], typ: GLenum) -> Result<u32> {
        unsafe {
            let shader = gl::CreateShader(typ);
            gl::ShaderSource(shader, 1, &(src.as_ptr() as _), ptr::null_mut());
            gl::CompileShader(shader);

            let mut res = 0;
            let mut info_log = [0u8; 512];
            let mut info_len = 0;

            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut res);

            if res == 0 {
                gl::GetShaderInfoLog(
                    shader,
                    info_log.len() as _,
                    &mut info_len as _,
                    info_log.as_mut_ptr() as _,
                );
                let info_msg = String::from_utf8_lossy(&info_log);
                return Err(eyre!("Failed to compile a shader: '{}'", info_msg));
            }

            Ok(shader)
        }
    }

    /// Tries to create a shader program and then checks for linking errors.
    fn create_shader_program(shader_ids: &[ShaderId]) -> Result<ProgramId> {
        unsafe {
            let shader_program = gl::CreateProgram();
            for sid in shader_ids {
                gl::AttachShader(shader_program, *sid);
            }

            gl::LinkProgram(shader_program);

            Self::check_shader_program_errors(shader_program)?;

            for sid in shader_ids {
                gl::DeleteShader(*sid);
            }

            Ok(shader_program)
        }
    }

    fn check_shader_program_errors(shader_program: ProgramId) -> Result<()> {
        let mut res = 0;
        let mut info_log = [0u8; 512];
        let mut info_len = 0;

        unsafe {
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut res);

            if res == 0 {
                gl::GetProgramInfoLog(
                    shader_program,
                    info_log.len() as _,
                    &mut info_len as _,
                    info_log.as_mut_ptr() as _,
                );
                let info_msg = String::from_utf8_lossy(&info_log);
                return Err(eyre!("Failed to create a shader program: '{}'", info_msg));
            }
        }

        Ok(())
    }

    const IMPORT_STR: &'static str = "//#import ";

    fn handle_imports(src: &mut String) -> Result<()> {
        while let Some(i) = src.find(Self::IMPORT_STR) {
            let path = src[(i + Self::IMPORT_STR.len())..]
                .split_whitespace()
                .next()
                .ok_or(eyre!("invalid #import path"))?;

            let import_src =
                String::from_utf8(fs::read(path).wrap_err("Couldn't load the import file")?)?;

            src.replace_range(i..i + Self::IMPORT_STR.len(), "//");
            src.insert_str(i, &import_src);
        }

        Ok(())
    }

    fn handle_defines(src: &mut String, defines: &[&str]) -> Result<()> {
        if let Some(index) = src.find("//#defines") {
            for define in defines {
                src.insert_str(index, &format!("#define {}\n", define));
            }

            Ok(())
        } else {
            Err(eyre!("Couldn't find //#defines in shader source"))
        }
    }

    //
    // Uniform setters...
    //

    #[allow(unused)]
    pub fn set_mat4(&self, mat: Mat4, name: &CStr) {
        unsafe {
            let loc = gl::GetUniformLocation(self.program_id, name.as_ptr() as _);
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, mat.to_cols_array().as_ptr() as _);
        }
    }

    #[allow(unused)]
    pub fn set_mat4_arr(&self, mats: &[Mat4], name: &CStr) {
        let mats_flat: Vec<f32> = mats.iter().flat_map(|m| m.to_cols_array()).collect();

        unsafe {
            let loc = gl::GetUniformLocation(self.program_id, name.as_ptr() as _);
            gl::UniformMatrix4fv(loc, mats.len() as i32, gl::FALSE, mats_flat.as_ptr() as _);
        }
    }

    #[allow(unused)]
    pub fn set_vec3(&self, vec: Vec3, name: &CStr) {
        unsafe {
            let loc = gl::GetUniformLocation(self.program_id, name.as_ptr() as _);
            gl::Uniform3f(loc, vec.x, vec.y, vec.z);
        }
    }

    #[allow(unused)]
    pub fn set_vec4(&self, vec: Vec4, name: &CStr) {
        unsafe {
            let loc = gl::GetUniformLocation(self.program_id, name.as_ptr() as _);
            gl::Uniform4f(loc, vec.x, vec.y, vec.z, vec.w);
        }
    }

    #[allow(unused)]
    pub fn set_f32(&self, v: f32, name: &CStr) {
        unsafe {
            let loc = gl::GetUniformLocation(self.program_id, name.as_ptr() as _);
            gl::Uniform1f(loc, v);
        }
    }

    #[allow(unused)]
    pub fn set_u32(&self, v: u32, name: &CStr) {
        unsafe {
            let loc = gl::GetUniformLocation(self.program_id, name.as_ptr() as _);
            gl::Uniform1ui(loc, v);
        }
    }

    #[allow(unused)]
    pub fn set_i32(&self, v: i32, name: &CStr) {
        unsafe {
            let loc = gl::GetUniformLocation(self.program_id, name.as_ptr() as _);
            gl::Uniform1i(loc, v);
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.program_id) }
    }
}
