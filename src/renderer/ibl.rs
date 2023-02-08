use std::{fs::File, io::BufReader};

use cstr::cstr;
use eyre::Result;

use gl::types::GLenum;
use image::codecs::hdr;

use crate::brdf_raw::BrdfType;
use crate::ogl::shader::shader_permutations::ShaderDefines;
use crate::ogl::shader::Shader;
use crate::ogl::ssbo::Ssbo;
use crate::ogl::texture::GlTexture;
use crate::ogl::{gl_time_query, TextureId};

use shader_constants::CONSTS;

const CUBEMAP_FACES: u32 = 6;
const IRRADIANCE_MAP_SIZE: i32 = 64;
const PREFILTER_MAP_SIZE: i32 = 256;
const MEASURED_BRDF_MAP_SIZE: u32 = 128;
const BRDF_LUT_SIZE: i32 = 512;

pub struct IblEnv {
    pub cubemap_tex: GlTexture,
    pub irradiance_tex: GlTexture,
    pub prefilter_tex: GlTexture,
}

impl IblEnv {
    pub fn from_equimap_path(path: &str) -> Result<Self> {
        let cubemap_tex = Self::load_cubemap_from_equi(path)?;
        let irradiance_tex = Self::compute_irradiance_map(cubemap_tex.id)?;
        let prefilter_tex = Self::compute_prefilter_map(cubemap_tex.id)?;

        irradiance_tex.add_label(cstr!("irradiance map"));
        prefilter_tex.add_label(cstr!("prefilter map"));

        Ok(Self {
            cubemap_tex,
            irradiance_tex,
            prefilter_tex,
        })
    }

    /// Converts an HDR equirectangular map to a cubemap
    fn load_cubemap_from_equi(path: &str) -> Result<GlTexture> {
        let equimap = load_hdr_image(path)?;
        let equi_tex = Self::create_equi_texture(equimap);
        let cubemap_tex = Self::create_cubemap_texture(
            CONSTS.ibl.cubemap_size,
            gl::RGBA32F,
            CONSTS.ibl.cubemap_roughnes_levels,
        );
        let equi_to_cubemap_shader =
            Shader::comp_with_path("shaders_stitched/equi_to_cubemap.comp")?;

        cubemap_tex.add_label(cstr!("environment map"));

        gl_time_query("equimap to cubemap", || {
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

                Self::dispatch_compute_divide(
                    CONSTS.ibl.cubemap_size as _,
                    CONSTS.ibl.cubemap_size as _,
                    CUBEMAP_FACES,
                );
                gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);

                gl::GenerateTextureMipmap(cubemap_tex.id);
            });
        });

        Ok(cubemap_tex)
    }

    pub fn compute_ibl_brdf<const BINDING: u32>(
        brdf_ssbo: &Ssbo<BINDING>,
        cubemap: &GlTexture,
        brdf_type: BrdfType,
    ) -> Result<GlTexture> {
        let tex = Self::create_cubemap_texture(IRRADIANCE_MAP_SIZE, gl::RGBA32F, 1);
        // TODO: shader permutations abstraction for compute shaders
        let shader = Shader::comp_with_path_defines(
            "shaders_stitched/raw_brdf_integration.comp",
            &brdf_type.defines(),
        )?;

        gl_time_query("measured BRDF compute shader", || {
            shader.use_shader(|| unsafe {
                gl::BindTextureUnit(0, cubemap.id);
                brdf_ssbo.bind();
                gl::BindImageTexture(1, tex.id, 0, gl::TRUE, 0, gl::WRITE_ONLY, gl::RGBA32F);

                Self::dispatch_compute_divide(
                    MEASURED_BRDF_MAP_SIZE,
                    MEASURED_BRDF_MAP_SIZE,
                    CUBEMAP_FACES,
                );

                gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
            });
        });

        Ok(tex)
    }

    /// Computes the diffuse irradiance map from the cubemap
    fn compute_irradiance_map(cubemap_tex_id: TextureId) -> Result<GlTexture> {
        let irradiance_tex = Self::create_cubemap_texture(IRRADIANCE_MAP_SIZE, gl::RGBA32F, 1);
        let irradiance_shader = Shader::comp_with_path("shaders_stitched/irradiance.comp")?;

        gl_time_query("irradiance compute shader", || {
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

                Self::dispatch_compute_divide(
                    IRRADIANCE_MAP_SIZE as u32,
                    IRRADIANCE_MAP_SIZE as u32,
                    CUBEMAP_FACES,
                );

                gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
            });
        });

        Ok(irradiance_tex)
    }

    /// Computes the LD part of the specular integral
    fn compute_prefilter_map(cubemap_tex_id: TextureId) -> Result<GlTexture> {
        let size = PREFILTER_MAP_SIZE;
        let mip_levels = CONSTS.ibl.cubemap_roughnes_levels;
        let prefilter_tex = Self::create_cubemap_texture(size, gl::RGBA32F, mip_levels);

        let prefilter_shader = Shader::comp_with_path("shaders_stitched/prefilter.comp")?;
        gl_time_query("split_sum prefiltering", || {
            prefilter_shader.use_shader(|| unsafe {
                gl::BindTextureUnit(0, cubemap_tex_id);

                for lod in 0..CONSTS.ibl.cubemap_roughnes_levels {
                    let roughness = lod as f32 / (CONSTS.ibl.cubemap_roughnes_levels as f32 - 1.);
                    prefilter_shader.set_f32(roughness, cstr!("linearRoughness"));

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
                    let mip_size = (PREFILTER_MAP_SIZE as f32 * 0.5f32.powi(lod)) as i32;
                    Self::dispatch_compute_divide(mip_size as _, mip_size as _, CUBEMAP_FACES);
                    gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
                }
            });
        });

        Ok(prefilter_tex)
    }

    unsafe fn dispatch_compute_divide(x: u32, y: u32, z: u32) {
        gl::DispatchCompute(
            x / CONSTS.ibl.local_size_xy,
            y / CONSTS.ibl.local_size_xy,
            z / CONSTS.ibl.local_size_z,
        );
    }

    fn create_cubemap_texture(size: i32, internal_typ: GLenum, mip_levels: i32) -> GlTexture {
        let tex = GlTexture::new(gl::TEXTURE_CUBE_MAP);

        unsafe {
            gl::TextureStorage2D(tex.id, mip_levels, internal_typ, size, size);

            let clamp = gl::CLAMP_TO_EDGE as i32;
            gl::TextureParameteri(tex.id, gl::TEXTURE_WRAP_S, clamp);
            gl::TextureParameteri(tex.id, gl::TEXTURE_WRAP_T, clamp);
            gl::TextureParameteri(tex.id, gl::TEXTURE_WRAP_R, clamp);
            gl::TextureParameteri(
                tex.id,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
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
}

/// Computes the BRDF part of the specular integral
pub fn dfg_integration() -> Result<GlTexture> {
    let brdf_lut = GlTexture::new(gl::TEXTURE_2D);

    unsafe {
        gl::TextureStorage2D(brdf_lut.id, 1, gl::RG32F, BRDF_LUT_SIZE, BRDF_LUT_SIZE);

        gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    };

    let dfg_integration_shader = Shader::comp_with_path("shaders_stitched/dfg_integration.comp")?;

    gl_time_query("split_sum dfg integration", || {
        dfg_integration_shader.use_shader(|| unsafe {
            gl::BindImageTexture(0, brdf_lut.id, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RG32F);

            gl::DispatchCompute(
                BRDF_LUT_SIZE as u32 / CONSTS.ibl.local_size_xy,
                BRDF_LUT_SIZE as u32 / CONSTS.ibl.local_size_xy,
                1,
            );
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        });
    });

    Ok(brdf_lut)
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
