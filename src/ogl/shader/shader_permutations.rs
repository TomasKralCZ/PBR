use std::{collections::HashMap, fs, hash::Hash};

use eyre::{Context, Result};

use super::Shader;

/// A module for handling the permutations of a shader.
///
/// A PBR shader has to work with every combination of textures. For example
/// one model might only have a metallic-roughness texture while some other model might
/// also have an emissive texture).
///
/// The compilation of these shaders is currently done on-demand.
pub struct ShaderPermutations<T: ShaderDefines> {
    permutations: HashMap<T, Shader>,
    vs_src: String,
    fs_src: String,
}

impl<T: ShaderDefines> ShaderPermutations<T> {
    pub fn new(vs_path: &str, fs_path: &str) -> Result<Self> {
        let vs_src =
            String::from_utf8(fs::read(vs_path).wrap_err("Couldn't load the vertex shader file")?)?;

        let fs_src = String::from_utf8(
            fs::read(fs_path).wrap_err("Couldn't load the fragment shader file")?,
        )?;

        Ok(Self {
            permutations: HashMap::new(),
            vs_src,
            fs_src,
        })
    }

    pub fn get_shader(&mut self, defines: T) -> Result<&Shader> {
        if !self.permutations.contains_key(&defines) {
            let shader = self.compile_shader(&defines)?;
            self.permutations.insert(defines.clone(), shader);
        }

        Ok(&self.permutations.get(&defines).as_ref().unwrap())
    }

    fn compile_shader(&mut self, defines: &T) -> Result<Shader> {
        Shader::with_src_defines(
            self.vs_src.clone(),
            &defines.defines_vs(),
            self.fs_src.clone(),
            &defines.defines_fs(),
        )
    }
}

pub trait ShaderDefines: Eq + Hash + Clone {
    fn defines_fs(&self) -> Vec<&str> {
        vec![]
    }

    fn defines_vs(&self) -> Vec<&str> {
        vec![]
    }
}
