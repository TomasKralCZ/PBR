use std::{fs::File, io::BufReader};

use cstr::cstr;
use eyre::Result;

use gl::types::GLenum;
use image::codecs::hdr;

use crate::ogl::shader::Shader;
use crate::ogl::ssbo::Ssbo;
use crate::ogl::texture::GlTexture;
use crate::ogl::{self, TextureId};

const CUBEMAP_SIZE: i32 = 1024; // SYNC this with prefilter.comp resolution !
const CUBEMAP_FACES: u32 = 6;
const IRRADIANCE_MAP_SIZE: i32 = 64;
const PREFILTER_MAP_SIZE: i32 = 256;
const PREFILTER_MAP_ROUGHNES_LEVELS: i32 = 7; // SYNC this with pbr.MAX_REFLECTION_LOD ! (minus 1)
const BRDF_LUT_SIZE: i32 = 512;

pub struct Ibl {
    pub textures: IblTextures,
}

pub struct IblTextures {
    pub irradiance_tex_id: GlTexture,
    pub prefilter_tex_id: GlTexture,
    pub brdf_lut_id: GlTexture,
}

impl Ibl {
    pub fn from_cubemap(cubemap_tex_id: &GlTexture) -> Result<Self> {
        let textures = Self::compute_ibl(cubemap_tex_id.id)?;

        Ok(Self { textures })
    }

    pub fn compute_ibl_raw_brdf(brdf_ssbo: &Ssbo<{ ogl::BRDF_DATA_BINDING }>) -> Result<GlTexture> {
        // TODO(high): HACK
        let cubemap = load_cubemap_from_equi("resources/IBL/rustig_koppie_puresky_4k.hdr")?;

        let tex = create_cubemap_texture(IRRADIANCE_MAP_SIZE, gl::RGBA32F);
        let shader = Shader::comp_with_path("shaders/ibl/raw_brdf_integration.comp")?;

        shader.use_shader(|| unsafe {
            gl::BindTextureUnit(0, cubemap.id);
            brdf_ssbo.bind();
            gl::BindImageTexture(1, tex.id, 0, gl::TRUE, 0, gl::WRITE_ONLY, gl::RGBA32F);

            gl::DispatchCompute(
                IRRADIANCE_MAP_SIZE as _,
                IRRADIANCE_MAP_SIZE as _,
                CUBEMAP_FACES,
            );

            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        });

        Ok(tex)
    }

    fn compute_ibl(cubemap_tex_id: TextureId) -> Result<IblTextures> {
        let irradiance_tex_id = Self::compute_irradiance_map(cubemap_tex_id)?;
        let prefilter_tex_id = Self::compute_prefilter_map(cubemap_tex_id)?;
        let brdf_lut_id = Self::brdf_integration()?;

        let textures = IblTextures {
            irradiance_tex_id,
            prefilter_tex_id,
            brdf_lut_id,
        };

        Ok(textures)
    }

    /// Computes the diffuse irradiance map from the cubemap
    fn compute_irradiance_map(cubemap_tex_id: u32) -> Result<GlTexture> {
        let irradiance_tex = create_cubemap_texture(IRRADIANCE_MAP_SIZE, gl::RGBA32F);
        let irradiance_shader = Shader::comp_with_path("shaders/ibl/irradiance.comp")?;

        irradiance_shader.use_shader(|| unsafe {
            gl::BindTextureUnit(0, cubemap_tex_id);
            gl::BindImageTexture(
                1,
                irradiance_tex.id,
                0,
                gl::TRUE,
                0,
                gl::WRITE_ONLY,
                gl::RGBA32F,
            );

            gl::DispatchCompute(
                IRRADIANCE_MAP_SIZE as _,
                IRRADIANCE_MAP_SIZE as _,
                CUBEMAP_FACES,
            );

            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        });

        Ok(irradiance_tex)
    }

    /// Computes the BRDF part of the specular integral
    fn brdf_integration() -> Result<GlTexture> {
        let brdf_lut = GlTexture::new(gl::TEXTURE_2D);

        unsafe {
            gl::TextureStorage2D(brdf_lut.id, 1, gl::RG32F, BRDF_LUT_SIZE, BRDF_LUT_SIZE);

            gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        };

        let brdf_integration_shader = Shader::comp_with_path("shaders/ibl/brdf_integration.comp")?;

        brdf_integration_shader.use_shader(|| unsafe {
            gl::BindImageTexture(0, brdf_lut.id, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RG32F);

            gl::DispatchCompute(BRDF_LUT_SIZE as _, BRDF_LUT_SIZE as _, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        });

        Ok(brdf_lut)
    }

    /// Computes the LD part of the specular integral
    fn compute_prefilter_map(cubemap_tex_id: u32) -> Result<GlTexture> {
        let prefilter_tex = GlTexture::new(gl::TEXTURE_CUBE_MAP);

        unsafe {
            let size = PREFILTER_MAP_SIZE;
            let levels = PREFILTER_MAP_ROUGHNES_LEVELS;
            gl::TextureStorage2D(prefilter_tex.id, levels, gl::RGBA32F, size, size);

            let clamp = gl::CLAMP_TO_EDGE as i32;
            gl::TextureParameteri(prefilter_tex.id, gl::TEXTURE_WRAP_S, clamp);
            gl::TextureParameteri(prefilter_tex.id, gl::TEXTURE_WRAP_T, clamp);
            gl::TextureParameteri(prefilter_tex.id, gl::TEXTURE_WRAP_R, clamp);

            let filtering = gl::LINEAR_MIPMAP_LINEAR as i32;
            gl::TextureParameteri(prefilter_tex.id, gl::TEXTURE_MIN_FILTER, filtering);
            gl::TextureParameteri(prefilter_tex.id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }

        let prefilter_shader = Shader::comp_with_path("shaders/ibl/prefilter.comp")?;
        prefilter_shader.use_shader(|| unsafe {
            gl::BindTextureUnit(0, cubemap_tex_id);

            for lod in 0..PREFILTER_MAP_ROUGHNES_LEVELS {
                let roughness = lod as f32 / (PREFILTER_MAP_ROUGHNES_LEVELS as f32 - 1.);
                prefilter_shader.set_f32(roughness, cstr!("perceptualRoughness"));

                gl::BindImageTexture(
                    1,
                    prefilter_tex.id,
                    lod,
                    gl::TRUE,
                    0,
                    gl::WRITE_ONLY,
                    gl::RGBA32F,
                );

                // TODO: make sure this matches the atual mip sizes...
                let mip_width = (PREFILTER_MAP_SIZE as f32 * 0.5f32.powi(lod)) as i32;
                let mip_height = (PREFILTER_MAP_SIZE as f32 * 0.5f32.powi(lod)) as i32;

                gl::DispatchCompute(mip_width as _, mip_height as _, CUBEMAP_FACES);
                gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
            }
        });

        Ok(prefilter_tex)
    }
}

/// Converts an HDR equirectangular map to a cubemap
pub fn load_cubemap_from_equi(path: &str) -> Result<GlTexture> {
    let equimap = load_hdr_image(path)?;
    let equi_tex = create_equi_texture(equimap);
    let cubemap_tex = create_cubemap_texture(CUBEMAP_SIZE, gl::RGBA32F);
    let equi_to_cubemap_shader = Shader::comp_with_path("shaders/ibl/equi_to_cubemap.comp")?;

    equi_to_cubemap_shader.use_shader(|| unsafe {
        gl::BindTextureUnit(0, equi_tex.id);
        gl::BindImageTexture(
            1,
            cubemap_tex.id,
            0,
            gl::TRUE,
            0,
            gl::WRITE_ONLY,
            gl::RGBA32F,
        );

        gl::DispatchCompute(CUBEMAP_SIZE as _, CUBEMAP_SIZE as _, CUBEMAP_FACES);
        gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

        gl::GenerateTextureMipmap(cubemap_tex.id);
    });

    Ok(cubemap_tex)
}

fn create_cubemap_texture(size: i32, internal_typ: GLenum) -> GlTexture {
    let tex = GlTexture::new(gl::TEXTURE_CUBE_MAP);

    unsafe {
        gl::TextureStorage2D(tex.id, 1, internal_typ, size, size);

        let clamp = gl::CLAMP_TO_EDGE as i32;
        gl::TextureParameteri(tex.id, gl::TEXTURE_WRAP_S, clamp);
        gl::TextureParameteri(tex.id, gl::TEXTURE_WRAP_T, clamp);
        gl::TextureParameteri(tex.id, gl::TEXTURE_WRAP_R, clamp);
        gl::TextureParameteri(tex.id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TextureParameteri(tex.id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    tex
}

fn create_equi_texture(equimap: HdrImage) -> GlTexture {
    let equi_tex = GlTexture::new(gl::TEXTURE_2D);

    unsafe {
        let w = equimap.width as i32;
        let h = equimap.height as i32;

        gl::TextureStorage2D(equi_tex.id, 1, gl::RGB32F, w, h);
        gl::TextureSubImage2D(
            equi_tex.id,
            0,
            0,
            0,
            w,
            h,
            gl::RGB,
            gl::FLOAT,
            equimap.pixels.as_ptr() as _,
        );

        gl::TextureParameteri(equi_tex.id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TextureParameteri(equi_tex.id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TextureParameteri(equi_tex.id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TextureParameteri(equi_tex.id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    }

    equi_tex
}

struct HdrImage {
    pixels: Vec<f32>,
    width: u32,
    height: u32,
}

fn load_hdr_image(path: &str) -> Result<HdrImage> {
    let file = File::open(path)?;
    let file = BufReader::new(file);

    let decoder = hdr::HdrDecoder::new(file)?;
    let metadata = decoder.metadata();
    let (width, height) = (metadata.width, metadata.height);

    let pixels: Vec<f32> = decoder
        .read_image_hdr()?
        .chunks(width as usize)
        .rev()
        .flatten()
        .flat_map(|rgb| rgb.0)
        .collect();

    Ok(HdrImage {
        pixels,
        width,
        height,
    })
}
