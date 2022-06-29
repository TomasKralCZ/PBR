use std::mem::size_of;

use eyre::{eyre, Result};
use gl::types::GLenum;
use glam::{Vec2, Vec3};
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
///
// TODO: It's not needed to store all this data in RAM.
// TODO: load vertex data without allocation and copying
pub struct Primitive {
    /// OpenGL VAO identifier
    pub vao: u32,
    /// Vertex indices
    pub indices: Indices,
    /// Vertex positions
    pub positions: Vec<Vec3>,
    /// Vertex texture coordinates
    pub texcoords: Vec<Vec2>,
    /// Vertex normals
    pub normals: Vec<Vec3>,
    /// An albedo texture
    pub albedo_texture: Option<u32>,
    /// Metallic and rougness texture
    pub mr_texture: Option<u32>,
    /// Normal texture
    pub normal_texture: Option<u32>,
    /// Occlusion texture
    pub occlusion_texture: Option<u32>,
    /// Emissive texture
    pub emissive_texture: Option<u32>,
}

impl Primitive {
    /// Creates the primitive from the gltf::Primitive struct and the DataBundle
    pub fn from_gltf(primitive: &gltf::Primitive, bundle: &mut DataBundle) -> Result<Self> {
        let mode = primitive.mode();

        if mode != gltf::mesh::Mode::Triangles {
            return Err(eyre!("primitive mode: '{mode:?}' is not impelemnted"));
        }

        let reader = primitive.reader(|buffer| Some(&bundle.buffers[buffer.index()]));

        let positions = reader
            .read_positions()
            .ok_or(eyre!("primitive doesn't containt positions"))?
            .map(Vec3::from)
            .collect();

        let indices = match reader
            .read_indices()
            .ok_or(eyre!("primitive doesn't containt indices"))?
        {
            ReadIndices::U32(b) => Indices::U32(b.collect()),
            ReadIndices::U16(b) => Indices::U16(b.collect()),
            ReadIndices::U8(b) => Indices::U8(b.collect()),
        };

        let mut texcoords = Vec::new();
        let mut texture_set = 0;
        while let Some(texcoords_reader) = reader.read_tex_coords(texture_set) {
            if texture_set >= 1 {
                eprintln!("WARN: primitive has more than 1 texture coordinate set");
                break;
            }

            texcoords = texcoords_reader.into_f32().map(Vec2::from).collect();

            texture_set += 1;
        }

        let normals = reader
            .read_normals()
            .ok_or(eyre!("primitive doesn't containt normals"))?
            .map(Vec3::from)
            .collect();

        let material = primitive.material();

        let mut primitive = Self {
            vao: 0,
            indices,
            positions,
            texcoords,
            normals,
            albedo_texture: None,
            mr_texture: None,
            normal_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
        };

        primitive.create_buffers();
        primitive.create_textures(&material, bundle);

        if primitive.vao == 0 {
            return Err(eyre!("primitive VAO wasn't correctly initialized"));
        }

        Ok(primitive)
    }

    /// Creates the OpenGL buffer from the loaded vertex data
    fn create_buffers(&mut self) {
        let mut indices = 0;
        let mut vao = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let _positions = ogl::create_float_buf(&self.positions, 3, ogl::POS_INDEX, gl::FLOAT);
            let _texcoords =
                ogl::create_float_buf(&self.texcoords, 2, ogl::TEXCOORDS_INDEX, gl::FLOAT);
            let _normals = ogl::create_float_buf(&self.normals, 3, ogl::NORMALS_INDEX, gl::FLOAT);

            // Indices
            gl::GenBuffers(1, &mut indices);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indices);

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                self.indices.size() as isize,
                self.indices.ptr(),
                gl::STATIC_DRAW,
            );

            // Unbind buffers
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            self.vao = vao;
        }
    }

    fn create_textures(&mut self, material: &gltf::Material, bundle: &mut DataBundle) {
        let pbr = material.pbr_metallic_roughness();

        if let Some(tex_info) = pbr.base_color_texture() {
            if pbr.base_color_factor() != [1.0, 1.0, 1.0, 1.0] {
                // Maybe just multiply them here instead of dealing with it in the shaders ?
                todo!("Base color factor");
            }

            self.albedo_texture = Some(self.create_texture(&tex_info.texture(), bundle))
        };

        if let Some(tex_info) = pbr.metallic_roughness_texture() {
            if pbr.metallic_factor() != 1.0 || pbr.roughness_factor() != 1.0 {
                // Maybe just multiply them here instead of dealing with it in the shaders ?
                todo!("Metallic and roughness factors");
            }

            self.mr_texture = Some(self.create_texture(&tex_info.texture(), bundle))
        }

        if let Some(tex_info) = material.normal_texture() {
            if tex_info.scale() != 1.0 {
                // Maybe just multiply them here instead of dealing with it in the shaders ?
                todo!("Normal map scale");
            }

            self.normal_texture = Some(self.create_texture(&tex_info.texture(), bundle))
        }

        if let Some(tex_info) = material.occlusion_texture() {
            if tex_info.strength() != 1.0 {
                // Maybe just multiply them here instead of dealing with it in the shaders ?
                todo!("Occlusion texture strength");
            }

            self.occlusion_texture = Some(self.create_texture(&tex_info.texture(), bundle))
        }

        if let Some(tex_info) = material.emissive_texture() {
            if material.emissive_factor() != [1.0, 1.0, 1.0] {
                // Maybe just multiply them here instead of dealing with it in the shaders ?
                //todo!("Emissive texture factor");
            }

            self.emissive_texture = Some(self.create_texture(&tex_info.texture(), bundle))
        }
    }

    // TODO: refactor -> creating GL textures should be done elsewhere, maybe porting to Vulkan would be easier
    /// Creates a new OpenGL texture.
    ///
    /// If the texture already exists (bundle.gl_textures\[texture_index\] == Some(...)),
    /// no new texture is created, only the Texture struct is cloned.
    fn create_texture(&mut self, tex: &gltf::Texture, bundle: &mut DataBundle) -> u32 {
        let tex_index = tex.source().index();
        if let Some(texture) = bundle.gl_textures[tex_index].clone() {
            return texture;
        }

        let gl_tex_id = unsafe {
            let mut texture = 0;

            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            self.set_texture_sampler(&tex.sampler());

            let image = &bundle.images[tex_index];

            assert!(image.width.is_power_of_two());
            assert!(image.height.is_power_of_two());

            let (internal_format, format) = match image.format {
                Format::R8G8 => (gl::RG8, gl::RG),
                Format::R8G8B8 => (gl::RGB8, gl::RGB),
                Format::R8G8B8A8 => (gl::RGBA8, gl::RGBA),
                f => unimplemented!("Unimplemented image format: '{f:?}'"),
            };

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format as i32,
                image.width as i32,
                image.height as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                image.pixels.as_ptr() as _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            texture
        };

        bundle.gl_textures[tex_index] = Some(gl_tex_id);
        gl_tex_id
    }

    /// Sets the appropriate sampler functions for the currently created texture.
    fn set_texture_sampler(&self, sampler: &gltf::texture::Sampler) {
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
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter as i32);
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
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_s as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_t as i32);
        }
    }
}

/// Vertex indices for a primitive.
///
/// Better than using generics here.
pub enum Indices {
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
