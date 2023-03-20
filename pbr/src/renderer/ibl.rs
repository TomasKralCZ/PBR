use std::{cmp::Ordering, fs::File, io::BufReader};

use cstr::cstr;
use eyre::{eyre, Result};

use gl::types::GLenum;
use image::codecs::hdr;

use crate::ogl::{gl_time_query, shader::Shader, texture::GlTexture, TextureId};

use shader_constants::CONSTS;

const CUBEMAP_FACES: u32 = 6;
const IRRADIANCE_MAP_SIZE: i32 = 64;
const PREFILTER_MAP_SIZE: i32 = 256;
const BRDF_LUT_SIZE: i32 = 512;

pub struct IblEnv {
    pub cubemap_tex: GlTexture,
    pub irradiance_tex: GlTexture,
    pub prefilter_tex: GlTexture,
}

impl IblEnv {
    pub fn from_equimap_path(path: &str) -> Result<Self> {
        let (cubemap_tex, max_radiance) = Self::load_cubemap_from_equi(path)?;
        let irradiance_tex = Self::compute_irradiance_map(cubemap_tex.id, max_radiance)?;
        let prefilter_tex = Self::compute_prefilter_map(cubemap_tex.id, max_radiance)?;

        irradiance_tex.add_label(cstr!("irradiance map"));
        prefilter_tex.add_label(cstr!("prefilter map"));

        Ok(Self {
            cubemap_tex,
            irradiance_tex,
            prefilter_tex,
        })
    }

    /// Converts an HDR equirectangular map to a cubemap
    fn load_cubemap_from_equi(path: &str) -> Result<(GlTexture, f32)> {
        let equimap = load_hdr_image(path)?;
        let max_value = equimap.max_value;
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

        Ok((cubemap_tex, max_value))
    }

    /// Computes the diffuse irradiance map from the cubemap
    fn compute_irradiance_map(cubemap_tex_id: TextureId, max_radiance: f32) -> Result<GlTexture> {
        let irradiance_tex = Self::create_cubemap_texture(IRRADIANCE_MAP_SIZE, gl::RGBA32F, 1);
        let irradiance_shader = Shader::comp_with_path("shaders_stitched/irradiance.comp")?;

        println!("Computing the irradiance map...");

        let (sample_delta, max_time) = match max_radiance {
            x if x > 0. && x < 1000. => (0.01, "a few seconds"),
            x if x > 1000. && x < 50000. => (0.005, "up to 30 seconds"),
            _ => (0.0015, "more than a minute"),
        };

        println!(
            "Max radiance is: {}, setting sample_delta to: {}",
            max_radiance, sample_delta
        );

        println!(
            "WARNING: this MIGHT take {} and MAY make your PC unresponsive",
            max_time
        );

        gl_time_query("irradiance compute shader", || {
            irradiance_shader.use_shader(|| unsafe {
                irradiance_shader.set_f32(sample_delta, cstr!("sampleDelta"));

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

                // Split the compute invocations into smaller parts so the OS doesn't timeout the GPU
                const STEP_SIZE: u32 = 8;
                for offset_x in (0..IRRADIANCE_MAP_SIZE).step_by(STEP_SIZE as usize) {
                    irradiance_shader.set_u32(offset_x as u32, cstr!("offset_x"));

                    for offset_y in (0..IRRADIANCE_MAP_SIZE).step_by(STEP_SIZE as usize) {
                        irradiance_shader.set_u32(offset_y as u32, cstr!("offset_y"));

                        Self::dispatch_compute_divide(STEP_SIZE, STEP_SIZE, CUBEMAP_FACES);
                        gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
                        gl::Finish();
                    }
                }
            });
        });

        Ok(irradiance_tex)
    }

    /// Computes the LD part of the specular integral
    fn compute_prefilter_map(cubemap_tex_id: TextureId, max_radiance: f32) -> Result<GlTexture> {
        let size = PREFILTER_MAP_SIZE;
        let mip_levels = CONSTS.ibl.cubemap_roughnes_levels;
        let prefilter_tex = Self::create_cubemap_texture(size, gl::RGBA32F, mip_levels);

        println!("Computing the prefilter map...");

        let (num_samples, max_time) = match max_radiance {
            x if x > 0. && x < 1000. => (1024, "a few seconds"),
            x if x > 1000. && x < 50000. => (512 * 1024, "up to 30 seconds"),
            _ => (1024 * 1024, "more than a minute"),
        };

        println!(
            "Max radiance is: {}, setting num_samples to: {}",
            max_radiance, num_samples
        );

        println!(
            "WARNING: this MIGHT take {} and MAY make your PC unresponsive",
            max_time
        );

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

                    // Less rough mip levels don't need so many samples
                    let num_samples = match lod {
                        0 => 1024,
                        1 => num_samples / 4,
                        2 => num_samples / 2,
                        _ => num_samples,
                    }
                    .max(1024);

                    let mip_size = PREFILTER_MAP_SIZE / 2i32.pow(lod as u32);
                    prefilter_shader.set_i32(num_samples, cstr!("sampleCount"));

                    // Split the compute invocations into smaller parts so the OS doesn't timeout the GPU
                    const STEP_SIZE: u32 = 8;
                    for offset_x in (0..mip_size).step_by(STEP_SIZE as usize) {
                        prefilter_shader.set_u32(offset_x as u32, cstr!("offset_x"));

                        for offset_y in (0..mip_size).step_by(STEP_SIZE as usize) {
                            prefilter_shader.set_u32(offset_y as u32, cstr!("offset_y"));

                            Self::dispatch_compute_divide(STEP_SIZE, STEP_SIZE, CUBEMAP_FACES);
                            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
                            gl::Finish();
                        }
                    }
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
    max_value: f32,
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

    let max_value = {
        pixels
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Ordering::Equal))
            .ok_or(eyre!("Couldn't find max value"))?
    };

    let max_value = *max_value;

    Ok(HdrImage {
        pixels,
        width,
        height,
        max_value,
    })
}
