use crate::{
    brdf_raw::BrdfType,
    ogl::shader::{
        shader_permutations::{ShaderDefines, ShaderPermutations},
        Shader,
    },
    scene::Primitive,
};
use eyre::Result;

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

impl PbrDefines {
    pub fn from_prim(prim: &Primitive) -> Self {
        let pbr = &prim.pbr_material;
        let cc = prim.clearcoat.as_ref();

        Self {
            albedo_map: pbr.base_color_texture.is_some(),
            mr_map: pbr.mr_texture.is_some(),
            normal_map: pbr.normal_texture.is_some(),
            occlusion_map: pbr.occlusion_texture.is_some(),
            emissive_map: pbr.emissive_texture.is_some(),
            clearcoat_enabled: cc.is_some(),
            clearcoat_intensity_map: cc.and_then(|c| c.intensity_texture.as_ref()).is_some(),
            clearcoat_roughness_map: cc.and_then(|c| c.roughness_texture.as_ref()).is_some(),
            clearcoat_normal_map: cc.and_then(|c| c.normal_texture.as_ref()).is_some(),
            anisotropy_enabled: prim.anisotropy.is_some(),
        }
    }
}

impl ShaderDefines for PbrDefines {
    fn defines(&self) -> Vec<&str> {
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
            // TODO(high): work on anisotropy
            //(self.anisotropy_enabled, "ANISOTROPY"),
        ];

        for (field, define) in fiels_defines {
            if field {
                defines.push(define);
            }
        }

        defines
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct DataDrivenDefines {
    pub normal_map: bool,
    pub occlusion_map: bool,
    pub brdf_typ: BrdfType,
}

impl DataDrivenDefines {
    pub fn from_prim_brdf(prim: &Primitive, brdf_typ: BrdfType) -> Self {
        let pbr = &prim.pbr_material;

        Self {
            normal_map: pbr.normal_texture.is_some(),
            occlusion_map: pbr.occlusion_texture.is_some(),
            brdf_typ,
        }
    }
}

impl ShaderDefines for DataDrivenDefines {
    fn defines(&self) -> Vec<&str> {
        let mut defines = Vec::new();

        let fiels_defines = [
            (self.normal_map, "NORMAL_MAP"),
            (self.occlusion_map, "OCCLUSION_MAP"),
        ];

        for (field, define) in fiels_defines {
            if field {
                defines.push(define);
            }
        }

        defines.push(self.brdf_typ.to_str());

        defines
    }
}

pub struct Shaders {
    pub pbr_shaders: ShaderPermutations<PbrDefines>,
    pub data_based_shaders: ShaderPermutations<DataDrivenDefines>,
    pub light_shader: Shader,
    pub cubemap_shader: Shader,
}

impl Shaders {
    pub fn new() -> Result<Self> {
        let pbr_shaders =
            ShaderPermutations::new("shaders_stitched/basic.vert", "shaders_stitched/pbr.frag")?;
        let data_based_shaders = ShaderPermutations::new(
            "shaders_stitched/basic.vert",
            "shaders_stitched/data_driven.frag",
        )?;
        let light_shader =
            Shader::with_files("shaders_stitched/basic.vert", "shaders_stitched/light.frag")?;
        let cubemap_shader = Shader::with_files(
            "shaders_stitched/cubemap.vert",
            "shaders_stitched/cubemap.frag",
        )?;

        Ok(Self {
            pbr_shaders,
            data_based_shaders,
            light_shader,
            cubemap_shader,
        })
    }
}
