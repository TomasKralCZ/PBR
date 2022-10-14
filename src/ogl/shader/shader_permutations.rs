use std::fs;

use eyre::{Context, Result};

use super::Shader;

/// A module for handling the permutations of a shader.
///
/// A PBR shader has to work with every combination of textures. For example
/// one model might only have a metallic-roughness texture while some other model might
/// also have an emissive texture).
///
/// The compilation of these shaders is currently done on-demand.
pub struct ShaderPermutations {
    permutations: Vec<Option<Shader>>,
    vs_src: String,
    fs_src: String,
}

impl ShaderPermutations {
    pub fn new(num_defines: u32, vs_path: &str, fs_path: &str) -> Result<Self> {
        let mut vs_src =
            String::from_utf8(fs::read(vs_path).wrap_err("Couldn't load the vertex shader file")?)?;
        let mut fs_src = String::from_utf8(
            fs::read(fs_path).wrap_err("Couldn't load the fragment shader file")?,
        )?;

        vs_src.push('\0');
        fs_src.push('\0');

        Ok(Self {
            permutations: vec![None; 2usize.pow(num_defines)],
            vs_src,
            fs_src,
        })
    }

    pub fn get_shader(&mut self, defines: &[impl ShaderDefines]) -> Result<Shader> {
        let mut index = 0;

        // Get the index of the shader
        for define in defines {
            if define.is_active() {
                let bit = 1 << define.rank();
                index |= bit;
            }
        }

        if let Some(shader) = self.permutations[index] {
            Ok(shader)
        } else {
            let shader = self.compile_shader(defines)?;
            self.permutations[index] = Some(shader);
            Ok(shader)
        }
    }

    fn compile_shader(&mut self, defines: &[impl ShaderDefines]) -> Result<Shader> {
        let defines_str: Vec<&str> = defines
            .iter()
            .filter(|d| d.is_active())
            .map(|d| d.as_ref())
            .collect();

        Shader::with_src_defines(self.vs_src.clone(), &[], self.fs_src.clone(), &defines_str)
    }
}

pub trait ShaderDefines: AsRef<str> {
    fn is_active(&self) -> bool;
    fn rank(&self) -> u32;
}
