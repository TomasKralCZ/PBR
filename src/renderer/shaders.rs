use crate::ogl::shader::{
    shader_permutations::{ShaderDefines, ShaderPermutations},
    Shader,
};
use eyre::Result;

pub struct Shaders {
    pub pbr: ShaderPermutations,
    pub light_shader: Shader,
    pub cubemap_shader: Shader,
}

pub enum PbrDefine {
    Albedo(bool),
    Mr(bool),
    Normal(bool),
    Occlusion(bool),
    Emissive(bool),
}

impl AsRef<str> for PbrDefine {
    fn as_ref(&self) -> &str {
        match self {
            PbrDefine::Albedo(_) => "ALBEDO_MAP",
            PbrDefine::Mr(_) => "MR_MAP",
            PbrDefine::Normal(_) => "NORMAL_MAP",
            PbrDefine::Occlusion(_) => "OCCLUSION_MAP",
            PbrDefine::Emissive(_) => "EMISSIVE_MAP",
        }
    }
}

impl ShaderDefines for PbrDefine {
    fn is_active(&self) -> bool {
        match self {
            PbrDefine::Albedo(active) => *active,
            PbrDefine::Mr(active) => *active,
            PbrDefine::Normal(active) => *active,
            PbrDefine::Occlusion(active) => *active,
            PbrDefine::Emissive(active) => *active,
        }
    }

    fn rank(&self) -> u32 {
        match self {
            PbrDefine::Albedo(_) => 0,
            PbrDefine::Mr(_) => 1,
            PbrDefine::Normal(_) => 2,
            PbrDefine::Occlusion(_) => 3,
            PbrDefine::Emissive(_) => 4,
        }
    }
}

impl Shaders {
    pub fn new() -> Result<Self> {
        let pbr = ShaderPermutations::new(5, "shaders/basic.vert", "shaders/pbr.frag")?;
        let light_shader = Shader::with_files("shaders/basic.vert", "shaders/light.frag")?;
        let cubemap_shader = Shader::with_files("shaders/cubemap.vert", "shaders/cubemap.frag")?;

        Ok(Self {
            pbr,
            light_shader,
            cubemap_shader,
        })
    }
}
