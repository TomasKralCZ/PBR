use std::rc::Rc;

use cstr::cstr;
use eyre::Result;
use glam::{Mat4, Vec3};

use crate::{
    app::AppState,
    camera::Camera,
    ogl::{self, texture::GlTexture, uniform_buffer::UniformBuffer, vao::Vao},
    scene::{Mesh, Node, Primitive, Scene},
    window::AppWindow,
};

mod cubemap;
mod lighting;
pub mod material;
pub mod probe;
mod settings;
mod shaders;
mod transforms;

pub use material::PbrMaterial;

use self::{
    lighting::Lighting,
    probe::Probe,
    settings::Settings,
    shaders::{PbrDefines, Shaders},
    transforms::Transforms,
};

/// A component responsible for rendering the scene.
pub struct Renderer {
    pub viewport_dim: ViewportDim,
    pub shaders: Shaders,
    /// Current MVP transformation matrices
    transforms: UniformBuffer<Transforms>,
    /// Current mesh material
    pub material: UniformBuffer<PbrMaterial>,
    /// Current lighting settings
    lighting: UniformBuffer<Lighting>,
    /// Runtime rendering settings
    pub settings: UniformBuffer<Settings>,
    sphere: Scene,
    cube: Vao,
    cubemap: GlTexture,
    probe: Probe,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(window: &AppWindow) -> Result<Self> {
        let cubemap = probe::load_cubemap_from_equi("resources/IBL/rustig_koppie_puresky_4k.hdr")?;
        let probe = Probe::from_cubemap(&cubemap)?;

        Ok(Self {
            viewport_dim: ViewportDim::new(window),
            shaders: Shaders::new()?,
            transforms: UniformBuffer::new(Transforms::new_indentity()),
            material: UniformBuffer::new(PbrMaterial::new()),
            lighting: UniformBuffer::new(Lighting::new()),
            settings: UniformBuffer::new(Settings::new()),
            sphere: Scene::from_gltf("resources/Sphere.glb")?,
            cube: cubemap::init_cube(),
            cubemap,
            probe,
        })
    }

    /// Render a new frame
    pub fn render(&mut self, scene: &mut Scene, camera: &mut dyn Camera, appstate: &AppState) {
        self.reset_gl_state();
        self.update_uniforms(camera, scene);

        self.render_lights();

        let transform = scene.transform;
        self.render_node(&scene.root, transform, appstate);

        self.draw_cubemap();
    }

    fn update_uniforms(&mut self, camera: &mut dyn Camera, scene: &Scene) {
        self.settings.update();

        let persp = Mat4::perspective_rh_gl(
            f32::to_radians(60.),
            self.viewport_dim.width / self.viewport_dim.height,
            0.1,
            1000.,
        );

        self.transforms.inner.projection = persp;
        self.transforms.inner.view = camera.view_mat();
        self.transforms.inner.model = scene.transform;
        self.transforms.update();

        self.lighting.inner.cam_pos = camera.get_pos().extend(0.0);
        self.lighting.update();
    }

    fn draw_cubemap(&mut self) {
        self.shaders.cubemap_shader.use_shader(|| unsafe {
            gl::BindTextureUnit(0, self.cubemap.id);

            gl::BindVertexArray(self.cube.id);
            gl::DrawElements(
                gl::TRIANGLES,
                cubemap::INDICES.len() as _,
                gl::UNSIGNED_BYTE,
                0 as _,
            );
            gl::BindVertexArray(0);
        });
    }

    fn reset_gl_state(&self) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);

            gl::Enable(gl::MULTISAMPLE);

            gl::Viewport(
                self.viewport_dim.min_x as i32,
                self.viewport_dim.min_y as i32,
                self.viewport_dim.width as i32,
                self.viewport_dim.height as i32,
            );
            gl::ClearColor(0.15, 0.15, 0.15, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);

            // TODO: enable / disable alopha blending based on GLTF
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    /// Recursive - traverses the node hierarchy and handles each node.
    fn render_node(&mut self, node: &Node, outer_transform: Mat4, appstate: &AppState) {
        let next_level_transform = outer_transform * node.transform;

        if let Some(mesh) = &node.mesh {
            self.render_mesh(mesh, next_level_transform, appstate);
        }

        for node in &node.children {
            self.render_node(node, next_level_transform, appstate);
        }
    }

    /// Renders the mesh of a node
    fn render_mesh(&mut self, mesh: &Mesh, node_transform: Mat4, appstate: &AppState) {
        self.transforms.inner.model = node_transform;
        self.transforms.update();

        for primitive in &mesh.primitives {
            self.bind_textures(primitive);
            self.set_material(primitive, appstate);

            let defines = Self::prim_pbr_defines(primitive);
            let shader = self
                .shaders
                .pbr_shaders
                .get_shader(defines)
                .expect("Error getting a shader");

            shader.use_shader(|| {
                Self::draw_mesh(primitive);
            })
        }
    }

    fn bind_textures(&mut self, primitive: &Primitive) {
        let bind_texture_unit = |tex: &Option<Rc<GlTexture>>, port: u32| {
            if let Some(tex) = tex {
                unsafe {
                    gl::BindTextureUnit(port, tex.id);
                }
            }
        };

        bind_texture_unit(&primitive.pbr_material.base_color_texture, ogl::ALBEDO_PORT);
        bind_texture_unit(&primitive.pbr_material.mr_texture, ogl::MR_PORT);
        bind_texture_unit(&primitive.pbr_material.normal_texture, ogl::NORMAL_PORT);
        bind_texture_unit(
            &primitive.pbr_material.occlusion_texture,
            ogl::OCCLUSION_PORT,
        );
        bind_texture_unit(&primitive.pbr_material.emissive_texture, ogl::EMISSIVE_PORT);

        if let Some(cc) = &primitive.clearcoat {
            bind_texture_unit(&cc.intensity_texture, ogl::CLEARCOAT_INTENSITY_PORT);
            bind_texture_unit(&cc.roughness_texture, ogl::CLEARCOAT_ROUGHNESS_PORT);
            bind_texture_unit(&cc.normal_texture, ogl::CLEARCOAT_NORMAL_PORT);
        }

        unsafe {
            gl::BindTextureUnit(
                ogl::IRRADIANCE_PORT,
                self.probe.textures.irradiance_tex_id.id,
            );
            gl::BindTextureUnit(ogl::PREFILTER_PORT, self.probe.textures.prefilter_tex_id.id);
            gl::BindTextureUnit(ogl::BRDF_PORT, self.probe.textures.brdf_lut_id.id);
        }
    }

    fn prim_pbr_defines(prim: &Primitive) -> PbrDefines {
        let pbr = &prim.pbr_material;
        let cc = prim.clearcoat.as_ref();

        PbrDefines {
            albedo_map: pbr.base_color_texture.is_some(),
            mr_map: pbr.mr_texture.is_some(),
            normal_map: pbr.normal_texture.is_some(),
            occlusion_map: pbr.occlusion_texture.is_some(),
            emissive_map: pbr.emissive_texture.is_some(),
            clearcoat_enabled: cc.is_some(),
            clearcoat_intensity_map: cc.and_then(|c| c.intensity_texture.as_ref()).is_some(),
            clearcoat_roughness_map: cc.and_then(|c| c.roughness_texture.as_ref()).is_some(),
            clearcoat_normal_map: cc.and_then(|c| c.normal_texture.as_ref()).is_some(),
            anisotropy_enabled: prim.anisotropy.is_some(),
        }
    }

    fn set_material(&mut self, prim: &Primitive, appstate: &AppState) {
        if appstate.should_override_material {
            self.material.inner = appstate.pbr_material_override;
        } else {
            self.material.inner.base_color_factor = prim.pbr_material.base_color_factor;
            self.material.inner.emissive_factor[0..3]
                .copy_from_slice(&prim.pbr_material.emissive_factor);
            self.material.inner.metallic_factor = prim.pbr_material.metallic_factor;
            self.material.inner.roughness_factor = prim.pbr_material.roughness_factor;
            self.material.inner.normal_scale = prim.pbr_material.normal_scale;
            self.material.inner.occlusion_strength = prim.pbr_material.occlusion_strength;

            if let Some(intensity_factor) = prim.clearcoat.as_ref().map(|c| c.intensity_factor) {
                self.material.inner.clearcoat_intensity_factor = intensity_factor;
            }

            if let Some(roughness_factor) = prim.clearcoat.as_ref().map(|c| c.roughness_factor) {
                self.material.inner.clearcoat_roughness_factor = roughness_factor;
            }

            if let Some(normal_scale) = prim.clearcoat.as_ref().map(|c| c.normal_scale) {
                self.material.inner.clearcoat_normal_scale = normal_scale;
            }

            if let Some(anisotropy) = prim.anisotropy.as_ref().map(|a| a.anisotropy) {
                self.material.inner.anisotropy = anisotropy;
            }
        }

        self.material.update();
    }

    fn draw_mesh(prim: &Primitive) {
        unsafe {
            gl::BindVertexArray(prim.vao.id);

            gl::DrawElements(
                gl::TRIANGLES,
                prim.num_indices as i32,
                prim.indices_type,
                0 as _,
            );

            gl::BindVertexArray(0);
        };
    }

    fn render_lights(&mut self) {
        let lighting = self.lighting.inner;
        let num_lights = lighting.lights;

        let prim = &self.sphere.root.children[0]
            .mesh
            .as_ref()
            .unwrap()
            .primitives[0];

        for (light_pos, light_color) in lighting
            .light_pos
            .iter()
            .zip(lighting.light_color)
            .take(num_lights as usize)
        {
            self.shaders.light_shader.use_shader(|| {
                self.transforms.inner.model = Mat4::from_translation(light_pos.truncate())
                    * Mat4::from_scale(Vec3::splat(0.1));
                self.transforms.update();

                self.shaders
                    .light_shader
                    .set_vec3(light_color.truncate(), cstr!("lightColor"));

                Self::draw_mesh(prim);
            });
        }
    }
}

pub struct ViewportDim {
    pub min_x: f32,
    pub min_y: f32,
    pub width: f32,
    pub height: f32,
}

impl ViewportDim {
    pub fn new(window: &AppWindow) -> Self {
        let width = window.width as f32;
        let height = window.height as f32;

        Self {
            min_x: 0.,
            min_y: 0.,
            width,
            height,
        }
    }
}
