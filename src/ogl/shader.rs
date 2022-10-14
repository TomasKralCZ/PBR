use eyre::{eyre, Context, Result};
use gl::types::GLenum;
use glam::{Mat4, Vec3, Vec4};
use std::{fs, ptr};

pub mod shader_permutations;

type ShaderId = u32;

/// Represents an OpenGL shader.
///
/// Allows setting uniforms with set_<> methods.
///
/// Use the `render` method for draw calls.
#[derive(Clone, Copy)]
pub struct Shader {
    pub shader_id: ShaderId,
}

impl Shader {
    /// Use this shader to render.
    ///
    /// Draw calls should be passed using the `render` function parameter.
    pub fn draw_with<F>(&self, render: F)
    where
        F: FnOnce(),
    {
        unsafe {
            gl::UseProgram(self.shader_id);

            render();

            gl::UseProgram(0);
        }
    }

    /// Loads a vertex shader and a fragment shader from specified paths and tries to create a shader program
    pub fn with_files(vs_path: &str, fs_path: &str) -> Result<Shader> {
        let mut vs_src =
            String::from_utf8(fs::read(vs_path).wrap_err("Couldn't load the vertex shader file")?)?;
        let mut fs_src = String::from_utf8(
            fs::read(fs_path).wrap_err("Couldn't load the fragment shader file")?,
        )?;

        // Add null-terminators !
        vs_src.push('\0');
        fs_src.push('\0');

        Self::with_src_defines(vs_src, &[], fs_src, &[])
    }

    /// Loads a vertex shader and a fragment shader from specified paths and tries to create a shader program
    pub fn with_src_defines(
        mut vs_src: String,
        vs_defines: &[&str],
        mut fs_src: String,
        fs_defines: &[&str],
    ) -> Result<Shader> {
        if !vs_defines.is_empty() {
            Self::handle_defines(&mut vs_src, vs_defines)?;
        }

        if !fs_defines.is_empty() {
            Self::handle_defines(&mut fs_src, fs_defines)?;
        }

        let vs = Self::compile_shader_glsl(vs_src.as_bytes(), gl::VERTEX_SHADER)
            .wrap_err_with(|| format!("Error compiling shader, defines: '{:?}'", fs_defines))?;
        let fs = Self::compile_shader_glsl(fs_src.as_bytes(), gl::FRAGMENT_SHADER)
            .wrap_err_with(|| format!("Error compiling shader, defines: '{:?}'", fs_defines))?;
        let shader_id = Self::link_shaders(vs, fs)?;
        Ok(Shader { shader_id })
    }

    fn handle_defines(src: &mut String, defines: &[&str]) -> Result<()> {
        if let Some(index) = src.find("//@DEFINES@") {
            for define in defines {
                src.insert_str(index, &format!("#define {}\n", define));
            }

            Ok(())
        } else {
            Err(eyre!("Couldn't find //@DEFINES@ in shader source"))
        }
    }

    #[allow(dead_code)]
    pub fn with_spv(vs_path: &str, fs_path: &str) -> Result<Self> {
        let vs_src = fs::read(vs_path).wrap_err("Couldn't load the vertex shader file")?;
        let fs_src = fs::read(fs_path).wrap_err("Couldn't load the fragment shader file")?;

        let vs = Self::compile_shader_spv(&vs_src, gl::VERTEX_SHADER)?;
        let fs = Self::compile_shader_spv(&fs_src, gl::FRAGMENT_SHADER)?;
        let shader_id = Self::link_shaders(vs, fs)
            .wrap_err_with(|| format!("Couldn't link shaders '{}', '{}'", vs_path, fs_path))?;
        Ok(Shader { shader_id })
    }

    /// Tries to compile a shader and checks for compilation errors.
    #[allow(dead_code)]
    fn compile_shader_spv(src: &[u8], typ: GLenum) -> Result<u32> {
        unsafe {
            let shader = gl::CreateShader(typ);
            gl::ShaderBinary(
                1,
                &shader,
                gl::SHADER_BINARY_FORMAT_SPIR_V,
                src.as_ptr() as _,
                src.len() as _,
            );
            gl::SpecializeShader(shader, "main\0".as_ptr() as _, 0, 0 as _, 0 as _);

            let mut res = 0;
            let mut info_log = [0u8; 512];
            let mut info_len = 0;

            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut res);

            if res == 0 {
                gl::GetShaderInfoLog(shader, 512, &mut info_len as _, info_log.as_mut_ptr() as _);
                let info_msg = String::from_utf8_lossy(&info_log);
                return Err(eyre!("Failed to compile a shader: '{}'", info_msg));
            }

            Ok(shader)
        }
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
                gl::GetShaderInfoLog(shader, 512, &mut info_len as _, info_log.as_mut_ptr() as _);
                let info_msg = String::from_utf8_lossy(&info_log);
                return Err(eyre!("Failed to compile a shader: '{}'", info_msg));
            }

            Ok(shader)
        }
    }

    /// Tries to link the vertex and fragment shaders (passed by their ids) and checks for linking errors.
    fn link_shaders(vs: ShaderId, fs: ShaderId) -> Result<ShaderId> {
        unsafe {
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vs);
            gl::AttachShader(shader_program, fs);
            gl::LinkProgram(shader_program);

            let mut res = 0;
            let mut info_log = [0u8; 512];
            let mut info_len = 0;

            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut res);

            if res == 0 {
                gl::GetProgramInfoLog(
                    shader_program,
                    512,
                    &mut info_len as _,
                    info_log.as_mut_ptr() as *mut i8,
                );
                let info_msg = String::from_utf8_lossy(&info_log);
                return Err(eyre!("Failed to create a shader program: '{}'", info_msg));
            }

            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            Ok(shader_program)
        }
    }

    //
    // Uniform setters...
    //

    #[allow(unused)]
    pub fn set_mat4(&self, mat: Mat4, name: &str) {
        Self::check_uniform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, mat.to_cols_array().as_ptr() as _);
        }
    }

    #[allow(unused)]
    pub fn set_mat4_arr(&self, mats: &[Mat4], name: &str) {
        Self::check_uniform_name(name);

        let mats_flat: Vec<f32> = mats.iter().flat_map(|m| m.to_cols_array()).collect();

        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::UniformMatrix4fv(loc, mats.len() as i32, gl::FALSE, mats_flat.as_ptr() as _);
        }
    }

    #[allow(unused)]
    pub fn set_vec3(&self, vec: Vec3, name: &str) {
        Self::check_uniform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::Uniform3f(loc, vec.x, vec.y, vec.z);
        }
    }

    #[allow(unused)]
    pub fn set_vec4(&self, vec: Vec4, name: &str) {
        Self::check_uniform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::Uniform4f(loc, vec.x, vec.y, vec.z, vec.w);
        }
    }

    #[allow(unused)]
    pub fn set_f32(&self, v: f32, name: &str) {
        Self::check_uniform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::Uniform1f(loc, v);
        }
    }

    #[allow(unused)]
    pub fn set_u32(&self, v: u32, name: &str) {
        Self::check_uniform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::Uniform1ui(loc, v);
        }
    }

    #[allow(unused)]
    pub fn set_i32(&self, v: i32, name: &str) {
        Self::check_uniform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::Uniform1i(loc, v);
        }
    }

    // TODO: use cstr!() macro...
    /// Uniform names have to be null-terminated and have to be ASCII (I think...)
    fn check_uniform_name(name: &str) {
        assert!(name.is_ascii());
        assert!(name.ends_with('\0'));
    }
}
