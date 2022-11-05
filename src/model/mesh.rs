use std::mem::size_of;

use eyre::{eyre, Result};
use gl::types::GLenum;
use gltf::{
    image::Format,
    mesh::util::ReadIndices,
    texture::{MagFilter, MinFilter, WrappingMode},
};

use crate::ogl::{self, TextureId};

mod tangents;

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

    pub pbr_material: StdPbrMaterial,
    pub clearcoat: Option<Clearcoat>,
}

impl Primitive {
    /// Creates the primitive from the gltf::Primitive struct and the DataBundle
    pub fn from_gltf(primitive: &gltf::Primitive, bundle: &mut DataBundle) -> Result<Self> {
        let mode = primitive.mode();
        if mode != gltf::mesh::Mode::Triangles {
            return Err(eyre!("primitive mode: '{mode:?}' is not impelemnted"));
        }

        let mut vertex_buf = Self::load_vertex_atrrib_buf(&primitive, bundle)?;
        let index_buf = Self::load_indices_buf(&primitive, bundle)?;

        let mut prim = Self {
            vao: 0,
            // The type is fixed no, maybe I'll revert it to flexible type in the future
            indices_type: gl::UNSIGNED_INT,
            num_indices: index_buf.len(),
            pbr_material: StdPbrMaterial::default(),
            clearcoat: None,
        };

        let material = primitive.material();
        prim.load_material(&material, bundle);

        // TODO: investigate trangent calculation condition (anisotropy as well, right ?)
        if prim.pbr_material.normal_texture.is_some()
            || prim
                .clearcoat
                .as_ref()
                .map(|c| c.normal_texture.is_some())
                .unwrap_or(false)
        {
            prim.calculate_tangents(&mut vertex_buf, &index_buf);
        }

        prim.vao = Self::create_vao(vertex_buf, index_buf);

        Ok(prim)
    }

    /// Creates OpenGL buffers from the loaded vertex data
    fn create_vao(vertex_buf: Vec<Vertex>, index_buf: Vec<u32>) -> u32 {
        let mut ibo = 0;
        let mut vao = 0;

        unsafe {
            gl::CreateVertexArrays(1, &mut vao);

            ogl::attach_float_buf_multiple_attribs(
                vao,
                &vertex_buf,
                &Vertex::ATTRIB_SIZES,
                &Vertex::ATTRIB_INDICES,
                &Vertex::ATTRIB_TYPES,
                size_of::<Vertex>(),
                &Vertex::ATTRIB_OFFSETS,
            );

            // Indices
            let indices_bytes = bytemuck::cast_slice::<u32, u8>(&index_buf);
            gl::CreateBuffers(1, &mut ibo);
            gl::NamedBufferStorage(
                ibo,
                indices_bytes.len() as isize,
                indices_bytes.as_ptr() as _,
                gl::DYNAMIC_STORAGE_BIT,
            );
            gl::VertexArrayElementBuffer(vao, ibo);
        }

        vao
    }

    fn load_vertex_atrrib_buf(
        primitive: &gltf::Primitive,
        bundle: &DataBundle,
    ) -> Result<Vec<Vertex>> {
        let reader = primitive.reader(|buffer| Some(&bundle.buffers[buffer.index()]));

        let mut position_iter = reader
            .read_positions()
            .ok_or(eyre!("primitive doesn't containt positions"))?;
        let mut normals_iter = reader
            .read_normals()
            .ok_or(eyre!("primitive doesn't containt normals"))?;

        // TODO: support models with more than 1 texture set ?
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

        let mut buf = Vec::with_capacity(position_iter.len());

        while let Some(pos) = position_iter.next() {
            let Some(normal) = normals_iter.next() else {
                return Err(eyre!("primitive attributes have different lengths"));
            };

            let texcoords = texcoords_reader
                .as_mut()
                .and_then(|t| t.next())
                .unwrap_or([0.; 2]);

            // I'm doing all the tangent computation myself for compatibility (gltf 2.0 tangents are in mikktspace)
            let tangent = [0.; 4];
            let vertex = Vertex {
                pos,
                normal,
                texcoords,
                tangent,
            };

            buf.push(vertex);
        }

        Ok(buf)
    }

    fn load_indices_buf(primitive: &gltf::Primitive, bundle: &DataBundle) -> Result<Vec<u32>> {
        let reader = primitive.reader(|buffer| Some(&bundle.buffers[buffer.index()]));

        let indices: Vec<u32> = match reader
            .read_indices()
            .ok_or(eyre!("primitive doesn't containt indices"))?
        {
            ReadIndices::U32(b) => b.collect(),
            ReadIndices::U16(b) => b.map(|i| i as u32).collect(),
            ReadIndices::U8(b) => b.map(|i| i as u32).collect(),
        };

        Ok(indices)
    }

    fn load_material(&mut self, material: &gltf::Material, bundle: &mut DataBundle) {
        let pbr = material.pbr_metallic_roughness();

        self.pbr_material.base_color_factor = pbr.base_color_factor();
        if let Some(tex_info) = pbr.base_color_texture() {
            self.pbr_material.base_color_texture =
                Some(self.create_texture(&tex_info.texture(), bundle))
        };

        self.pbr_material.metallic_factor = pbr.metallic_factor();
        self.pbr_material.roughness_factor = pbr.roughness_factor();
        if let Some(tex_info) = pbr.metallic_roughness_texture() {
            self.pbr_material.mr_texture = Some(self.create_texture(&tex_info.texture(), bundle))
        }

        if let Some(normal_tex_info) = material.normal_texture() {
            self.pbr_material.normal_scale = normal_tex_info.scale();
            self.pbr_material.normal_texture =
                Some(self.create_texture(&normal_tex_info.texture(), bundle))
        }

        if let Some(occlusion_texture) = material.occlusion_texture() {
            self.pbr_material.occlusion_strength = occlusion_texture.strength();
            self.pbr_material.occlusion_texture =
                Some(self.create_texture(&occlusion_texture.texture(), bundle))
        }

        self.pbr_material.emissive_factor = material.emissive_factor();
        if let Some(tex_info) = material.emissive_texture() {
            self.pbr_material.emissive_texture =
                Some(self.create_texture(&tex_info.texture(), bundle))
        }

        if let Some(clearcoat) = material.clearcoat() {
            let clearcoat_factor = clearcoat.clearcoat_factor();
            // The clearcoat layer is disabled if clearcoat == 0.0
            if clearcoat_factor != 0. {
                let mut clearcoat_m = Clearcoat::default();

                clearcoat_m.intensity_factor = clearcoat_factor;
                if let Some(clearcoat_texture) = clearcoat.clearcoat_texture() {
                    clearcoat_m.intensity_texture =
                        Some(self.create_texture(&clearcoat_texture.texture(), bundle))
                }

                clearcoat_m.roughness_factor = clearcoat.clearcoat_roughness_factor();
                if let Some(clearcoat_roughness_texture) = clearcoat.clearcoat_roughness_texture() {
                    clearcoat_m.roughness_texture =
                        Some(self.create_texture(&clearcoat_roughness_texture.texture(), bundle));
                }

                if let Some(clearcoat_normal_texture) = clearcoat.clearcoat_normal_texture() {
                    clearcoat_m.normal_texture_scale = clearcoat_normal_texture.scale();
                    clearcoat_m.normal_texture =
                        Some(self.create_texture(&clearcoat_normal_texture.texture(), bundle));
                }

                self.clearcoat = Some(clearcoat_m);
            }
        }
    }

    /// Creates a new OpenGL texture.
    ///
    /// If the texture already exists (bundle.gl_textures\[texture_index\] == Some(...)),
    /// no new texture is created, only the Texture struct is cloned.
    fn create_texture(&mut self, tex: &gltf::Texture, bundle: &mut DataBundle) -> TextureId {
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

impl Default for StdPbrMaterial {
    fn default() -> Self {
        Self {
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
        }
    }
}

pub struct Clearcoat {
    pub intensity_factor: f32,
    pub intensity_texture: Option<TextureId>,

    pub roughness_factor: f32,
    pub roughness_texture: Option<TextureId>,

    pub normal_texture: Option<TextureId>,
    pub normal_texture_scale: f32,
}

impl Default for Clearcoat {
    fn default() -> Self {
        Self {
            intensity_factor: 0.,
            intensity_texture: None,
            roughness_factor: 0.,
            roughness_texture: None,
            normal_texture: None,
            normal_texture_scale: 1.,
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub texcoords: [f32; 2],
    pub tangent: [f32; 4],
}

impl Vertex {
    const ATTRIB_SIZES: [i32; 4] = [3, 3, 2, 4];
    const ATTRIB_INDICES: [u32; 4] = [
        ogl::POSITION_INDEX,
        ogl::NORMALS_INDEX,
        ogl::TEXCOORDS_INDEX,
        ogl::TANGENT_INDEX,
    ];

    const ATTRIB_TYPES: [GLenum; 4] = [gl::FLOAT; 4];
    const ATTRIB_OFFSETS: [usize; 4] = [0, 12, 24, 32];
}
