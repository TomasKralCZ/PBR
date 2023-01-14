use crate::{scene::DataBundle, ogl::TextureId};

use super::create_texture;

/// Standard PBR material parameters
pub struct StdPbrMaterial {
    pub base_color_texture: Option<TextureId>,
    pub base_color_factor: [f32; 4],

    pub mr_texture: Option<TextureId>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,

    pub normal_texture: Option<TextureId>,
    pub normal_scale: f32,

    pub occlusion_texture: Option<TextureId>,
    pub occlusion_strength: f32,

    pub emissive_texture: Option<TextureId>,
    pub emissive_factor: [f32; 3],
}

impl StdPbrMaterial {
    pub fn from_gtlf(material: &gltf::Material, bundle: &mut DataBundle) -> Self {
        let pbr = material.pbr_metallic_roughness();

        let base_color_factor = pbr.base_color_factor();
        let base_color_texture = pbr
            .base_color_texture()
            .map(|tex_info| create_texture(&tex_info.texture(), bundle));

        let metallic_factor = pbr.metallic_factor();
        let roughness_factor = pbr.roughness_factor();
        let mr_texture = pbr
            .metallic_roughness_texture()
            .map(|tex_info| create_texture(&tex_info.texture(), bundle));

        let normal_scale = material
            .normal_texture()
            .map(|tex_info| tex_info.scale())
            .unwrap_or(1.0);

        let normal_texture = material
            .normal_texture()
            .map(|tex_info| create_texture(&tex_info.texture(), bundle));

        let occlusion_strength = material
            .occlusion_texture()
            .map(|tex_info| tex_info.strength())
            .unwrap_or(1.0);

        let occlusion_texture = material
            .occlusion_texture()
            .map(|tex_info| create_texture(&tex_info.texture(), bundle));

        let emissive_factor = material.emissive_factor();
        let emissive_texture = material
            .emissive_texture()
            .map(|tex_info| create_texture(&tex_info.texture(), bundle));

        Self {
            base_color_texture,
            base_color_factor,
            mr_texture,
            metallic_factor,
            roughness_factor,
            normal_texture,
            normal_scale,
            occlusion_texture,
            occlusion_strength,
            emissive_texture,
            emissive_factor,
        }
    }
}

/// Clearcoat extension parameters
pub struct Clearcoat {
    pub intensity_factor: f32,
    pub intensity_texture: Option<TextureId>,

    pub roughness_factor: f32,
    pub roughness_texture: Option<TextureId>,

    pub normal_texture: Option<TextureId>,
    pub normal_scale: f32,
}

impl Clearcoat {
    pub fn from_gltf(cc: &gltf::material::Clearcoat, bundle: &mut DataBundle) -> Option<Self> {
        let intensity_factor = cc.clearcoat_factor();
        // The clearcoat layer is disabled if clearcoat == 0.0
        if intensity_factor != 0. {
            let intensity_texture = cc
                .clearcoat_texture()
                .map(|tex_info| create_texture(&tex_info.texture(), bundle));

            let roughness_factor = cc.clearcoat_roughness_factor();
            let roughness_texture = cc
                .clearcoat_roughness_texture()
                .map(|tex_info| create_texture(&tex_info.texture(), bundle));

            let normal_scale = cc
                .clearcoat_normal_texture()
                .map(|tex_info| tex_info.scale())
                .unwrap_or(1.0);

            let normal_texture = cc
                .clearcoat_normal_texture()
                .map(|tex_info| create_texture(&tex_info.texture(), bundle));

            return Some(Self {
                intensity_factor,
                intensity_texture,
                roughness_factor,
                roughness_texture,
                normal_texture,
                normal_scale,
            });
        }

        None
    }
}

/// Simple anisotropy parameter.
/// This is a placeholder until real anisotropy extension is stabilized in gltf 2.0.
pub struct Anisotropy {
    pub anisotropy: f32,
}

impl Anisotropy {
    pub fn new() -> Self {
        Self { anisotropy: 0. }
    }
}
