use crate::ogl::shader::{
    shader_permutations::{ShaderDefines, ShaderPermutations},
    Shader,
};
use eyre::Result;

pub struct Shaders {
    pub pbr_shaders: ShaderPermutations<PbrDefines>,
    pub light_shader: Shader,
    pub cubemap_shader: Shader,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct PbrDefines {
    pub albedo_map: bool,
    pub mr_map: bool,
    pub normal_map: bool,
    pub occlusion_map: bool,
    pub emissive_map: bool,

    pub clearcoat_enabled: bool,
    pub clearcoat_intensity_map: bool,
    pub clearcoat_roughness_map: bool,
    pub clearcoat_normal_map: bool,

    pub anisotropy_enabled: bool,
}

impl ShaderDefines for PbrDefines {
    fn defines_fs(&self) -> Vec<&str> {
        let mut defines = Vec::new();

        let fiels_defines = [
            (self.albedo_map, "ALBEDO_MAP"),
            (self.mr_map, "MR_MAP"),
            (self.normal_map, "NORMAL_MAP"),
            (self.occlusion_map, "OCCLUSION_MAP"),
            (self.emissive_map, "EMISSIVE_MAP"),
            (self.clearcoat_enabled, "CLEARCOAT"),
            (self.clearcoat_intensity_map, "CLEARCOAT_INTENSITY_MAP"),
            (self.clearcoat_roughness_map, "CLEARCOAT_ROUGHNESS_MAP"),
            (self.clearcoat_normal_map, "CLEARCOAT_NORMAL_MAP"),
            (self.anisotropy_enabled, "ANISOTROPY"),
        ];

        for (field, define) in fiels_defines {
            if field {
                defines.push(define);
            }
        }

        defines
    }
}

impl Shaders {
    pub fn new() -> Result<Self> {
        let pbr = ShaderPermutations::new("shaders/basic.vert", "shaders/pbr.frag")?;
        let light_shader = Shader::with_files("shaders/basic.vert", "shaders/light.frag")?;
        let cubemap_shader = Shader::with_files("shaders/cubemap.vert", "shaders/cubemap.frag")?;

        Ok(Self {
            pbr_shaders: pbr,
            light_shader,
            cubemap_shader,
        })
    }
}
