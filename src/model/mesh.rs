use std::mem::size_of;

use bytemuck::offset_of;
use eyre::{eyre, Result};
use gl::types::GLenum;
use gltf::{
    image::Format,
    mesh::util::ReadIndices,
    texture::{MagFilter, MinFilter, WrappingMode},
};

use crate::ogl;

use super::DataBundle;

/// Gltf terminology is needlessly confusing.
/// A gltf 'Mesh' contains multiple real sub-meshes (called Primitives in the gltf parlance)
pub struct Mesh {
    /// 'Primitives' of the 'mesh'
    // TODO: could be optimized - most meshes probably only contain a single primitive - avoid allocating a vector
    pub primitives: Vec<Primitive>,
    /// Name of the 'Mesh'
    pub name: Option<String>,
}

impl Mesh {
    /// Create a mesh from the gltf::Mesh struct and the DataBundle
    pub fn from_gltf(mesh: &gltf::Mesh, bundle: &mut DataBundle) -> Result<Self> {
        let name = mesh.name().map(|n| n.to_owned());

        let mut primitives = Vec::new();
        for primitive in mesh.primitives() {
            let primitive = Primitive::from_gltf(&primitive, bundle)?;
            primitives.push(primitive);
        }

        Ok(Mesh { primitives, name })
    }
}

/// A Primitive represents a single 'mesh' in the normal meaning of that word
/// (a collection of vertices with a specific topology like Triangles or Lines).
pub struct Primitive {
    /// OpenGL VAO identifier
    pub vao: u32,
    pub indices_type: GLenum,
    pub num_indices: usize,
    pub base_color_texture: Option<u32>,
    pub base_color_factor: [f32; 4],
    pub mr_texture: Option<u32>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_texture: Option<u32>,
    pub normal_scale: f32,
    pub occlusion_texture: Option<u32>,
    pub occlusion_strength: f32,
    pub emissive_texture: Option<u32>,
    pub emissive_factor: [f32; 3],
}

impl Primitive {
    /// Creates the primitive from the gltf::Primitive struct and the DataBundle
    pub fn from_gltf(primitive: &gltf::Primitive, bundle: &mut DataBundle) -> Result<Self> {
        let mode = primitive.mode();
        if mode != gltf::mesh::Mode::Triangles {
            return Err(eyre!("primitive mode: '{mode:?}' is not impelemnted"));
        }

        let reader = primitive.reader(|buffer| Some(&bundle.buffers[buffer.index()]));

        let position_iter = reader
            .read_positions()
            .ok_or(eyre!("primitive doesn't containt positions"))?;
        let normals_iter = reader
            .read_normals()
            .ok_or(eyre!("primitive doesn't containt normals"))?;

        // TODO: handle textureless models...
        let mut texcoords_reader = None;
        let mut texture_set = 0;
        while let Some(reader) = reader.read_tex_coords(texture_set) {
            if texture_set >= 1 {
                eprintln!("WARN: primitive has more than 1 texture coordinate set");
                break;
            }

            texcoords_reader = Some(reader.into_f32());

            texture_set += 1;
        }

        let mut vertices = Vec::with_capacity(position_iter.len());
        for (pos, normal) in position_iter.zip(normals_iter) {
            let texcoords = texcoords_reader
                .as_mut()
                .and_then(|r| r.next())
                .unwrap_or([0.; 2]);
            vertices.push(Vertex {
                pos,
                normal,
                texcoords,
            });
        }

        let indices = match reader
            .read_indices()
            .ok_or(eyre!("primitive doesn't containt indices"))?
        {
            ReadIndices::U32(b) => Indices::U32(b.collect()),
            ReadIndices::U16(b) => Indices::U16(b.collect()),
            ReadIndices::U8(b) => Indices::U8(b.collect()),
        };

        let material = primitive.material();

        let mut primitive = Self {
            vao: 0,
            indices_type: indices.gl_type(),
            num_indices: indices.len(),
            base_color_texture: None,
            base_color_factor: [1.; 4],
            mr_texture: None,
            metallic_factor: 1.,
            roughness_factor: 1.,
            normal_texture: None,
            normal_scale: 1.,
            occlusion_texture: None,
            occlusion_strength: 1.,
            emissive_texture: None,
            emissive_factor: [0.; 3],
        };

        primitive.create_buffers(vertices, indices);
        primitive.create_textures(&material, bundle);

        Ok(primitive)
    }

    /// Creates OpenGL buffers from the loaded vertex data
    fn create_buffers(&mut self, vertices: Vec<Vertex>, indices: Indices) {
        let mut ibo = 0;
        let mut vao = 0;

        unsafe {
            gl::CreateVertexArrays(1, &mut vao);

            ogl::attach_float_buf_multiple_attribs(
                vao,
                &vertices,
                &[3, 3, 2],
                &[
                    ogl::POSITION_INDEX,
                    ogl::NORMALS_INDEX,
                    ogl::TEXCOORDS_INDEX,
                ],
                &[gl::FLOAT, gl::FLOAT, gl::FLOAT],
                size_of::<Vertex>(),
                &[
                    offset_of!(Vertex, pos),
                    offset_of!(Vertex, normal),
                    offset_of!(Vertex, texcoords),
                ],
            );

            // Indices
            gl::CreateBuffers(1, &mut ibo);
            gl::NamedBufferData(ibo, indices.size() as isize, indices.ptr(), gl::STATIC_DRAW);
            gl::VertexArrayElementBuffer(vao, ibo);

            self.vao = vao;
        }
    }

    fn create_textures(&mut self, material: &gltf::Material, bundle: &mut DataBundle) {
        let pbr = material.pbr_metallic_roughness();

        self.base_color_factor = pbr.base_color_factor();
        if let Some(tex_info) = pbr.base_color_texture() {
            self.base_color_texture = Some(self.create_texture(&tex_info.texture(), bundle))
        };

        self.metallic_factor = pbr.metallic_factor();
        self.roughness_factor = pbr.roughness_factor();
        if let Some(tex_info) = pbr.metallic_roughness_texture() {
            self.mr_texture = Some(self.create_texture(&tex_info.texture(), bundle))
        }

        if let Some(normal_tex_info) = material.normal_texture() {
            self.normal_scale = normal_tex_info.scale();
            if self.normal_scale != 1. {
                println!("Watch out for normal scale !");
            }
            self.normal_texture = Some(self.create_texture(&normal_tex_info.texture(), bundle))
        }

        if let Some(occlusion_texture) = material.occlusion_texture() {
            self.occlusion_strength = occlusion_texture.strength();
            self.occlusion_texture = Some(self.create_texture(&occlusion_texture.texture(), bundle))
        }

        self.emissive_factor = material.emissive_factor();
        if let Some(tex_info) = material.emissive_texture() {
            self.emissive_texture = Some(self.create_texture(&tex_info.texture(), bundle))
        }
    }

    /// Creates a new OpenGL texture.
    ///
    /// If the texture already exists (bundle.gl_textures\[texture_index\] == Some(...)),
    /// no new texture is created, only the Texture struct is cloned.
    fn create_texture(&mut self, tex: &gltf::Texture, bundle: &mut DataBundle) -> u32 {
        let tex_index = tex.source().index();
        if let Some(texture) = bundle.gl_textures[tex_index] {
            return texture;
        }

        let gl_tex_id = unsafe {
            let mut texture = 0;
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);

            self.set_texture_sampler(texture, &tex.sampler());

            let image = &bundle.images[tex_index];

            assert!(image.width.is_power_of_two());
            assert!(image.height.is_power_of_two());

            let (internal_format, format) = match image.format {
                Format::R8 => (gl::R8, gl::RED),
                Format::R8G8 => (gl::RG8, gl::RG),
                Format::R8G8B8 => (gl::RGB8, gl::RGB),
                Format::R8G8B8A8 => (gl::RGBA8, gl::RGBA),
                f => unimplemented!("Unimplemented image format: '{f:?}'"),
            };

            let w = image.width as i32;
            let h = image.height as i32;

            let levels = 1 + f32::floor(f32::log2(i32::max(w, h) as f32)) as i32;
            gl::TextureStorage2D(texture, levels, internal_format, w, h);
            gl::TextureSubImage2D(
                texture,
                0,
                0,
                0,
                w,
                h,
                format,
                gl::UNSIGNED_BYTE,
                image.pixels.as_ptr() as _,
            );

            gl::GenerateTextureMipmap(texture);

            texture
        };

        bundle.gl_textures[tex_index] = Some(gl_tex_id);
        gl_tex_id
    }

    /// Sets the appropriate sampler functions for the currently created texture.
    fn set_texture_sampler(&self, texture: u32, sampler: &gltf::texture::Sampler) {
        let min_filter = match sampler.min_filter() {
            Some(min_filter) => match min_filter {
                MinFilter::Nearest => gl::NEAREST,
                MinFilter::Linear => gl::LINEAR,
                MinFilter::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
                MinFilter::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
                MinFilter::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
                MinFilter::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
            },
            None => gl::LINEAR_MIPMAP_LINEAR,
        };

        let mag_filter = match sampler.mag_filter() {
            Some(mag_filter) => match mag_filter {
                MagFilter::Nearest => gl::NEAREST,
                MagFilter::Linear => gl::LINEAR,
            },
            None => gl::LINEAR,
        };

        unsafe {
            gl::TextureParameteri(texture, gl::TEXTURE_MIN_FILTER, min_filter as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_MAG_FILTER, mag_filter as i32);
        }

        let wrap_s = match sampler.wrap_s() {
            WrappingMode::ClampToEdge => gl::CLAMP_TO_EDGE,
            WrappingMode::MirroredRepeat => gl::MIRRORED_REPEAT,
            WrappingMode::Repeat => gl::REPEAT,
        };

        let wrap_t = match sampler.wrap_t() {
            WrappingMode::ClampToEdge => gl::CLAMP_TO_EDGE,
            WrappingMode::MirroredRepeat => gl::MIRRORED_REPEAT,
            WrappingMode::Repeat => gl::REPEAT,
        };

        unsafe {
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_S, wrap_s as i32);
            gl::TextureParameteri(texture, gl::TEXTURE_WRAP_T, wrap_t as i32);
        }
    }
}

/// Vertex indices for a primitive.
///
/// Better than using generics here.
enum Indices {
    U32(Vec<u32>),
    U16(Vec<u16>),
    U8(Vec<u8>),
}

impl Indices {
    /// The size (in bytes) of the buffer
    pub fn size(&self) -> usize {
        match self {
            Indices::U32(buf) => buf.len() * size_of::<u32>(),
            Indices::U16(buf) => buf.len() * size_of::<u16>(),
            Indices::U8(buf) => buf.len() * size_of::<u8>(),
        }
    }

    /// The lenght (in elements) of the buffer
    pub fn len(&self) -> usize {
        match self {
            Indices::U32(buf) => buf.len(),
            Indices::U16(buf) => buf.len(),
            Indices::U8(buf) => buf.len(),
        }
    }

    /// A pointer to the start of the buffer
    pub fn ptr(&self) -> *const std::ffi::c_void {
        match self {
            Indices::U32(buf) => buf.as_ptr() as _,
            Indices::U16(buf) => buf.as_ptr() as _,
            Indices::U8(buf) => buf.as_ptr() as _,
        }
    }

    /// A GL_TYPE corresponding to the variant of the buffer
    pub fn gl_type(&self) -> GLenum {
        match self {
            Indices::U32(_) => gl::UNSIGNED_INT,
            Indices::U16(_) => gl::UNSIGNED_SHORT,
            Indices::U8(_) => gl::UNSIGNED_BYTE,
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub texcoords: [f32; 2],
}
