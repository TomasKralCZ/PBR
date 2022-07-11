use std::{fs::File, io::BufReader};

use eyre::Result;

use glam::{vec3, Mat4};
use image::codecs::hdr;

use crate::ogl::shader::Shader;

use super::Renderer;

const CUBEMAP_SIZE: i32 = 2048;
const IRRADIANCE_MAP_SIZE: i32 = 64;

pub fn load_cubemaps(cube_vao: u32) -> Result<(u32, u32)> {
    let equimap = load_hdr_image("resources/IBL/PaperMill_Ruins_E/PaperMill_E_3k.hdr")?;

    let mut equi_tex_id = 0;
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

    let mut cubemap_tex_id = 0;
    let mut capture_fbo = 0;
    let mut capture_rbo = 0;
    unsafe {
        gl::CreateFramebuffers(1, &mut capture_fbo);
        gl::CreateRenderbuffers(1, &mut capture_rbo);

        gl::NamedRenderbufferStorage(
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
        );

        gl::CreateTextures(gl::TEXTURE_CUBE_MAP, 1, &mut cubemap_tex_id);
        gl::TextureStorage2D(cubemap_tex_id, 1, gl::RGB16F, CUBEMAP_SIZE, CUBEMAP_SIZE);

        let clamp = gl::CLAMP_TO_EDGE as i32;
        gl::TextureParameteri(cubemap_tex_id, gl::TEXTURE_WRAP_S, clamp);
        gl::TextureParameteri(cubemap_tex_id, gl::TEXTURE_WRAP_T, clamp);
        gl::TextureParameteri(cubemap_tex_id, gl::TEXTURE_WRAP_R, clamp);
        gl::TextureParameteri(cubemap_tex_id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TextureParameteri(cubemap_tex_id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
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

    let mut irradiance_tex_id = 0;
    unsafe {
        gl::CreateTextures(gl::TEXTURE_CUBE_MAP, 1, &mut irradiance_tex_id);

        gl::TextureStorage2D(
            irradiance_tex_id,
            1,
            gl::RGB16F,
            IRRADIANCE_MAP_SIZE,
            IRRADIANCE_MAP_SIZE,
        );

        let clamp = gl::CLAMP_TO_EDGE as i32;
        gl::TextureParameteri(irradiance_tex_id, gl::TEXTURE_WRAP_S, clamp);
        gl::TextureParameteri(irradiance_tex_id, gl::TEXTURE_WRAP_T, clamp);
        gl::TextureParameteri(irradiance_tex_id, gl::TEXTURE_WRAP_R, clamp);
        gl::TextureParameteri(irradiance_tex_id, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TextureParameteri(irradiance_tex_id, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    unsafe {
        gl::NamedRenderbufferStorage(
            capture_rbo,
            gl::DEPTH_COMPONENT24,
            IRRADIANCE_MAP_SIZE,
            IRRADIANCE_MAP_SIZE,
        );
    }

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

    Ok((cubemap_tex_id, irradiance_tex_id))
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
