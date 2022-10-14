use std::{fs::File, io::BufReader};

use eyre::Result;

use glam::{vec3, Mat4};
use image::codecs::hdr;

use crate::ogl::shader::Shader;

use super::Renderer;

const CUBEMAP_SIZE: i32 = 1024; // SYNC this with prefilter.frag resolution !
const IRRADIANCE_MAP_SIZE: i32 = 64;
const PREFILTER_MAP_SIZE: i32 = 256;
const PREFILTER_MAP_ROUGHNES_LEVELS: i32 = 7; // SYNC this with pbr MAX_REFLECTION_LOD ! (minus 1)
const BRDF_LUT_SIZE: i32 = 512;

pub fn load_cubemaps(cube_vao: u32, quad_vao: u32) -> Result<(u32, u32, u32, u32)> {
    //let equimap = load_hdr_image("resources/IBL/PaperMill_Ruins_E/PaperMill_E_3k.hdr")?;
    let equimap = load_hdr_image("resources/IBL/Hamarikyu_Bridge_B/14-Hamarikyu_Bridge_B_3k.hdr")?;

    let equi_tex_id = create_equi_texture(equimap);

    let cubemap_tex_id = create_cubemap_texture(CUBEMAP_SIZE);
    let mut capture_fbo = 0;
    /* let mut capture_rbo = 0; */
    unsafe {
        gl::CreateFramebuffers(1, &mut capture_fbo);
        /* gl::CreateRenderbuffers(1, &mut capture_rbo); */

        /* gl::NamedRenderbufferStorage(
            capture_rbo,
            gl::DEPTH_COMPONENT24,
            CUBEMAP_SIZE,
            CUBEMAP_SIZE,
        );
        gl::NamedFramebufferRenderbuffer(
            capture_fbo,
            gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER,
            capture_rbo,
        ); */
    }

    let capture_proj = Mat4::perspective_rh_gl(90f32.to_radians(), 1., 0.1, 10.);
    let capture_views = [
        Mat4::look_at_rh(vec3(0., 0., 0.), vec3(1., 0., 0.), vec3(0., -1., 0.)),
        Mat4::look_at_rh(vec3(0., 0., 0.), vec3(-1., 0., 0.), vec3(0., -1., 0.)),
        Mat4::look_at_rh(vec3(0., 0., 0.), vec3(0., 1., 0.), vec3(0., 0., 1.)),
        Mat4::look_at_rh(vec3(0., 0., 0.), vec3(0., -1., 0.), vec3(0., 0., -1.)),
        Mat4::look_at_rh(vec3(0., 0., 0.), vec3(0., 0., 1.), vec3(0., -1., 0.)),
        Mat4::look_at_rh(vec3(0., 0., 0.), vec3(0., 0., -1.), vec3(0., -1., 0.)),
    ];

    let equi_shader = Shader::with_files("shaders/equi.vert", "shaders/equi.frag")?;

    equi_shader.draw_with(|| unsafe {
        equi_shader.set_mat4(capture_proj, "projection\0");

        gl::BindTextureUnit(0, equi_tex_id);

        gl::Viewport(0, 0, CUBEMAP_SIZE, CUBEMAP_SIZE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, capture_fbo);

        for (i, view) in capture_views.iter().enumerate() {
            equi_shader.set_mat4(*view, "view\0");

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                cubemap_tex_id,
                0,
            );

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            Renderer::draw_cube(cube_vao);
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    });

    let equi_convolution_shader =
        Shader::with_files("shaders/equi.vert", "shaders/equi_convolution.frag")?;

    let irradiance_tex_id = create_cubemap_texture(IRRADIANCE_MAP_SIZE);

    /* unsafe {
        gl::NamedRenderbufferStorage(
            capture_rbo,
            gl::DEPTH_COMPONENT24,
            IRRADIANCE_MAP_SIZE,
            IRRADIANCE_MAP_SIZE,
        );
    } */

    equi_convolution_shader.draw_with(|| unsafe {
        equi_convolution_shader.set_mat4(capture_proj, "projection\0");
        gl::BindTextureUnit(0, cubemap_tex_id);

        gl::Viewport(0, 0, IRRADIANCE_MAP_SIZE, IRRADIANCE_MAP_SIZE);
        gl::BindFramebuffer(gl::FRAMEBUFFER, capture_fbo);

        for (i, view) in capture_views.iter().enumerate() {
            equi_convolution_shader.set_mat4(*view, "view\0");

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                irradiance_tex_id,
                0,
            );

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            Renderer::draw_cube(cube_vao);
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    });

    /*
        Prefiltering
    */
    let mut prefilter_tex_id = 0u32;

    unsafe {
        gl::CreateTextures(gl::TEXTURE_CUBE_MAP, 1, &mut prefilter_tex_id);

        let size = PREFILTER_MAP_SIZE;
        let levels = PREFILTER_MAP_ROUGHNES_LEVELS;
        gl::TextureStorage2D(prefilter_tex_id, levels, gl::RGB16F, size, size);

        let clamp = gl::CLAMP_TO_EDGE as i32;
        gl::TextureParameteri(prefilter_tex_id, gl::TEXTURE_WRAP_S, clamp);
        gl::TextureParameteri(prefilter_tex_id, gl::TEXTURE_WRAP_T, clamp);
        gl::TextureParameteri(prefilter_tex_id, gl::TEXTURE_WRAP_R, clamp);

        let filtering = gl::LINEAR_MIPMAP_LINEAR as i32;
        gl::TextureParameteri(prefilter_tex_id, gl::TEXTURE_MIN_FILTER, filtering);
        gl::TextureParameteri(prefilter_tex_id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    let prefilter_shader = Shader::with_files("shaders/equi.vert", "shaders/prefilter.frag")?;

    unsafe {
        gl::BindTextureUnit(0, cubemap_tex_id);

        gl::BindFramebuffer(gl::FRAMEBUFFER, capture_fbo);

        let levels = PREFILTER_MAP_ROUGHNES_LEVELS;
        for mip in 0..levels {
            // resize framebuffer according to mip-level size.
            let mip_width = (PREFILTER_MAP_SIZE as f32 * 0.5f32.powi(mip)) as i32;
            let mip_height = (PREFILTER_MAP_SIZE as f32 * 0.5f32.powi(mip)) as i32;

            //glBindRenderbuffer(GL_RENDERBUFFER, captureRBO);
            //glRenderbufferStorage(GL_RENDERBUFFER, GL_DEPTH_COMPONENT24, mipWidth, mipHeight);

            gl::Viewport(0, 0, mip_width, mip_height);

            let roughness = mip as f32 / (PREFILTER_MAP_ROUGHNES_LEVELS as f32 - 1.);

            prefilter_shader.draw_with(|| {
                prefilter_shader.set_mat4(capture_proj, "projection\0");
                prefilter_shader.set_f32(roughness, "roughness\0");

                for i in 0..6 {
                    prefilter_shader.set_mat4(capture_views[i], "view\0");

                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0,
                        gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                        prefilter_tex_id,
                        mip,
                    );

                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    Renderer::draw_cube(cube_vao);
                }
            });
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    let mut brdf_lut_id = 0;
    unsafe {
        gl::CreateTextures(gl::TEXTURE_2D, 1, &mut brdf_lut_id);

        gl::TextureStorage2D(brdf_lut_id, 1, gl::RGB16F, BRDF_LUT_SIZE, BRDF_LUT_SIZE);

        gl::TextureParameteri(brdf_lut_id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TextureParameteri(brdf_lut_id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TextureParameteri(brdf_lut_id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TextureParameteri(brdf_lut_id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

        brdf_lut_id
    };

    let brdf_shader = Shader::with_files(
        "shaders/brdf_integration.vert",
        "shaders/brdf_integration.frag",
    )?;

    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, capture_fbo);

        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            brdf_lut_id,
            0,
        );

        gl::Viewport(0, 0, BRDF_LUT_SIZE, BRDF_LUT_SIZE);

        brdf_shader.draw_with(|| {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            Renderer::draw_quad(quad_vao);
        });

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    Ok((
        cubemap_tex_id,
        irradiance_tex_id,
        prefilter_tex_id,
        brdf_lut_id,
    ))
}

fn create_cubemap_texture(size: i32) -> u32 {
    let mut cubemap_tex_id = 0u32;

    unsafe {
        gl::CreateTextures(gl::TEXTURE_CUBE_MAP, 1, &mut cubemap_tex_id);
        gl::TextureStorage2D(cubemap_tex_id, 1, gl::RGB16F, size, size);

        let clamp = gl::CLAMP_TO_EDGE as i32;
        gl::TextureParameteri(cubemap_tex_id, gl::TEXTURE_WRAP_S, clamp);
        gl::TextureParameteri(cubemap_tex_id, gl::TEXTURE_WRAP_T, clamp);
        gl::TextureParameteri(cubemap_tex_id, gl::TEXTURE_WRAP_R, clamp);
        gl::TextureParameteri(cubemap_tex_id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TextureParameteri(cubemap_tex_id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    cubemap_tex_id
}

fn create_equi_texture(equimap: HdrImage) -> u32 {
    let mut equi_tex_id = 0u32;

    unsafe {
        gl::CreateTextures(gl::TEXTURE_2D, 1, &mut equi_tex_id);

        let w = equimap.width as i32;
        let h = equimap.height as i32;

        gl::TextureStorage2D(equi_tex_id, 1, gl::RGB16F, w, h);
        gl::TextureSubImage2D(
            equi_tex_id,
            0,
            0,
            0,
            w,
            h,
            gl::RGB,
            gl::FLOAT,
            equimap.pixels.as_ptr() as _,
        );

        gl::TextureParameteri(equi_tex_id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TextureParameteri(equi_tex_id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TextureParameteri(equi_tex_id, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TextureParameteri(equi_tex_id, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    }

    equi_tex_id
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

    let pixels = decoder
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
