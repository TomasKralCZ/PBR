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

use shader_constants::IBL;

const CUBEMAP_FACES: u32 = 6;
const IRRADIANCE_MAP_SIZE: i32 = 64;
const PREFILTER_MAP_SIZE: i32 = 256;
const MEASURED_BRDF_MAP_SIZE: u32 = 128;
const BRDF_LUT_SIZE: i32 = 256;

pub struct Ibl {
    pub textures: IblTextures,
}

pub struct IblTextures {
    pub irradiance_tex_id: GlTexture,
    pub prefilter_tex_id: GlTexture,
    pub dfg_lut_id: GlTexture,
}

impl Ibl {
    pub fn from_cubemap(cubemap_tex_id: &GlTexture) -> Result<Self> {
        let textures = Self::compute_ibl(cubemap_tex_id.id)?;

        Ok(Self { textures })
    }

    pub fn compute_ibl_brdf<const BINDING: u32>(
        brdf_ssbo: &Ssbo<BINDING>,
        cubemap: &GlTexture,
        brdf_type: BrdfType,
    ) -> Result<GlTexture> {
        let tex = create_cubemap_texture(IRRADIANCE_MAP_SIZE, gl::RGBA32F);
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

    fn compute_ibl(cubemap_tex_id: TextureId) -> Result<IblTextures> {
        let irradiance_tex_id = Self::compute_irradiance_map(cubemap_tex_id)?;
        let prefilter_tex_id = Self::compute_prefilter_map(cubemap_tex_id)?;
        let dfg_lut_id = Self::dfg_integration()?;

        let textures = IblTextures {
            irradiance_tex_id,
            prefilter_tex_id,
            dfg_lut_id,
        };

        Ok(textures)
    }

    /// Computes the diffuse irradiance map from the cubemap
    fn compute_irradiance_map(cubemap_tex_id: u32) -> Result<GlTexture> {
        let irradiance_tex = create_cubemap_texture(IRRADIANCE_MAP_SIZE, gl::RGBA32F);
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

    /// Computes the BRDF part of the specular integral
    fn dfg_integration() -> Result<GlTexture> {
        let brdf_lut = GlTexture::new(gl::TEXTURE_2D);

        unsafe {
            gl::TextureStorage2D(brdf_lut.id, 1, gl::RG32F, BRDF_LUT_SIZE, BRDF_LUT_SIZE);

            gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TextureParameteri(brdf_lut.id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        };

        let dfg_integration_shader =
            Shader::comp_with_path("shaders_stitched/dfg_integration.comp")?;

        gl_time_query("split_sum dfg integration", || {
            dfg_integration_shader.use_shader(|| unsafe {
                gl::BindImageTexture(0, brdf_lut.id, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RG32F);

                gl::DispatchCompute(
                    BRDF_LUT_SIZE as u32 / IBL.local_size_xy,
                    BRDF_LUT_SIZE as u32 / IBL.local_size_xy,
                    1,
                );
                gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
            });
        });

        Ok(brdf_lut)
    }

    /// Computes the LD part of the specular integral
    fn compute_prefilter_map(cubemap_tex_id: u32) -> Result<GlTexture> {
        let prefilter_tex = GlTexture::new(gl::TEXTURE_CUBE_MAP);

        unsafe {
            let size = PREFILTER_MAP_SIZE;
            let levels = IBL.prefilter_map_roughnes_levels;
            gl::TextureStorage2D(prefilter_tex.id, levels, gl::RGBA32F, size, size);

            let clamp = gl::CLAMP_TO_EDGE as i32;
            gl::TextureParameteri(prefilter_tex.id, gl::TEXTURE_WRAP_S, clamp);
            gl::TextureParameteri(prefilter_tex.id, gl::TEXTURE_WRAP_T, clamp);
            gl::TextureParameteri(prefilter_tex.id, gl::TEXTURE_WRAP_R, clamp);

            let filtering = gl::LINEAR_MIPMAP_LINEAR as i32;
            gl::TextureParameteri(prefilter_tex.id, gl::TEXTURE_MIN_FILTER, filtering);
            gl::TextureParameteri(prefilter_tex.id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }

        let prefilter_shader = Shader::comp_with_path("shaders_stitched/prefilter.comp")?;
        gl_time_query("split_sum prefiltering", || {
            prefilter_shader.use_shader(|| unsafe {
                gl::BindTextureUnit(0, cubemap_tex_id);

                for lod in 0..IBL.prefilter_map_roughnes_levels {
                    let roughness = lod as f32 / (IBL.prefilter_map_roughnes_levels as f32 - 1.);
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
                    let mip_width = (PREFILTER_MAP_SIZE as f32 * 0.5f32.powi(lod)) as i32;
                    let mip_height = (PREFILTER_MAP_SIZE as f32 * 0.5f32.powi(lod)) as i32;

                    gl::DispatchCompute(mip_width as _, mip_height as _, CUBEMAP_FACES);
                    gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
                }
            });
        });

        Ok(prefilter_tex)
    }

    unsafe fn dispatch_compute_divide(x: u32, y: u32, z: u32) {
        gl::DispatchCompute(
            x / IBL.local_size_xy,
            y / IBL.local_size_xy,
            z / IBL.local_size_z,
        );
    }
}

/// Converts an HDR equirectangular map to a cubemap
pub fn load_cubemap_from_equi(path: &str) -> Result<GlTexture> {
    let equimap = load_hdr_image(path)?;
    let equi_tex = create_equi_texture(equimap);
    let cubemap_tex = create_cubemap_texture(IBL.cubemap_size, gl::RGBA32F);
    let equi_to_cubemap_shader = Shader::comp_with_path("shaders_stitched/equi_to_cubemap.comp")?;

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

        gl::DispatchCompute(IBL.cubemap_size as _, IBL.cubemap_size as _, CUBEMAP_FACES);
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
