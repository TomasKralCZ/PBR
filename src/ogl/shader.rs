use egui_inspect::EguiInspect;
use eyre::{eyre, Context, Result};
use gl::types::GLenum;
use glam::{Mat4, Vec3, Vec4};
use std::{fs, ptr};

/// Represents an OpenGL shader.
///
/// Allows setting uniforms with set_<> methods.
///
/// Use the `render` method for draw calls.
#[derive(EguiInspect)]
pub struct Shader {
    #[inspect(no_edit)]
    pub shader_id: u32,
    #[inspect(no_edit)]
    vs_path: String,
    #[inspect(no_edit)]
    fs_path: String,
}

impl Shader {
    /// Loads a vertex shader and a fragment shader from specified paths and tries to create a shader program
    pub fn with_file(vs_path: &str, fs_path: &str) -> Result<Shader> {
        let mut vs_src = fs::read(vs_path).wrap_err("Couldn't load the vertex shader file")?;
        let mut fs_src = fs::read(fs_path).wrap_err("Couldn't load the fragment shader file")?;

        // Add null-terminators
        vs_src.push(b'\0');
        fs_src.push(b'\0');

        let vs = Self::compile_shader(&vs_src, gl::VERTEX_SHADER)?;
        let fs = Self::compile_shader(&fs_src, gl::FRAGMENT_SHADER)?;
        let shader_id = Self::link_shaders(vs, fs)?;
        Ok(Shader {
            shader_id,
            vs_path: String::from(vs_path),
            fs_path: String::from(fs_path),
        })
    }

    pub fn reload(&mut self) -> Result<Self> {
        Self::with_file(&self.vs_path, &self.fs_path)
    }

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

    /// Tries to compile a shader and checks for compilation errors.
    fn compile_shader(src: &[u8], typ: GLenum) -> Result<u32> {
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
    fn link_shaders(vs: u32, fs: u32) -> Result<u32> {
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
        Self::check_inform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, mat.to_cols_array().as_ptr() as _);
        }
    }

    #[allow(unused)]
    pub fn set_mat4_arr(&self, mats: &[Mat4], name: &str) {
        Self::check_inform_name(name);

        let mats_flat: Vec<f32> = mats.iter().flat_map(|m| m.to_cols_array()).collect();

        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::UniformMatrix4fv(loc, mats.len() as i32, gl::FALSE, mats_flat.as_ptr() as _);
        }
    }

    #[allow(unused)]
    pub fn set_vec3(&self, vec: Vec3, name: &str) {
        Self::check_inform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::Uniform3f(loc, vec.x, vec.y, vec.z);
        }
    }

    #[allow(unused)]
    pub fn set_vec4(&self, vec: Vec4, name: &str) {
        Self::check_inform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::Uniform4f(loc, vec.x, vec.y, vec.z, vec.w);
        }
    }

    #[allow(unused)]
    pub fn set_f32(&self, v: f32, name: &str) {
        Self::check_inform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::Uniform1f(loc, v);
        }
    }

    #[allow(unused)]
    pub fn set_u32(&self, v: u32, name: &str) {
        Self::check_inform_name(name);
        unsafe {
            let loc = gl::GetUniformLocation(self.shader_id, name.as_ptr() as _);
            gl::Uniform1ui(loc, v);
        }
    }

    /// Uniform names have to be null-terminated and have to be ASCII (I think...)
    fn check_inform_name(name: &str) {
        assert!(name.is_ascii());
        assert!(name.ends_with('\0'));
    }
}
