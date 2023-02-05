use std::{cell::RefMut, rc::Rc};

use cstr::cstr;
use eyre::{eyre, Result};
use glam::{Mat4, Vec3};

use crate::{
    app_settings::{AppSettings, MaterialSrc},
    brdf_raw::{BrdfRaw, BrdfType},
    camera::Camera,
    ogl::{self, texture::GlTexture, uniform_buffer::UniformBuffer, vao::Vao},
    resources::Resources,
    scene::{Mesh, Node, Primitive, Scene},
    util::RcMut,
};

mod cubemap;
pub mod ibl;
mod lighting;
pub mod material;
pub mod pbr_settings;
mod shaders;
mod transforms;

pub use material::PbrMaterial;

use self::{
    ibl::Ibl,
    lighting::Lighting,
    pbr_settings::PbrSettings,
    shaders::{DataDrivenDefines, PbrDefines, Shaders},
    transforms::Transforms,
};

/// A component responsible for rendering the scene.
pub struct Renderer {
    app_settings: RcMut<AppSettings>,

    shaders: Shaders,
    /// Current MVP transformation matrices
    transforms: UniformBuffer<Transforms>,
    /// Current mesh material
    material: UniformBuffer<PbrMaterial>,
    /// Current lighting settings
    lighting: UniformBuffer<Lighting>,
    /// Runtime rendering settings
    pbr_settings: UniformBuffer<PbrSettings>,
    sphere: Scene,
    cube: Vao,
    cubemap: GlTexture,
    ibl: Ibl,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(app_settings: RcMut<AppSettings>) -> Result<Self> {
        // TODO: make this reloadable at runtime
        let cubemap = ibl::load_cubemap_from_equi("resources/IBL/rustig_koppie_puresky_4k.hdr")?;
        let ibl = Ibl::from_cubemap(&cubemap)?;

        Ok(Self {
            app_settings,
            shaders: Shaders::new()?,
            transforms: UniformBuffer::new(Transforms::new_indentity()),
            material: UniformBuffer::new(PbrMaterial::new()),
            lighting: UniformBuffer::new(Lighting::new()),
            pbr_settings: UniformBuffer::new(PbrSettings::new()),
            sphere: Scene::from_gltf("resources/gltf/Sphere.glb")?,
            cube: cubemap::init_cube(),
            cubemap,
            ibl,
        })
    }

    /// Render a new frame
    pub fn render(&mut self, camera: &mut dyn Camera, resources: RcMut<Resources>) -> Result<()> {
        self.reset_gl_state();
        self.update_uniforms(camera, resources.get_mut())?;

        self.render_lights()?;

        let selected_scene = self.app_settings.get().selected_scene;
        let mut res = resources.get_mut();
        let scene = res.get_scene(selected_scene)?;

        let transform = scene.transform;
        self.render_node(&scene.root, transform)?;

        self.draw_cubemap();

        Ok(())
    }

    fn update_uniforms(
        &mut self,
        camera: &mut dyn Camera,
        mut res: RefMut<Resources>,
    ) -> Result<()> {
        self.update_brdf(&mut res)?;

        let app_settings = self.app_settings.get();
        let selected_scene = app_settings.selected_scene;

        self.pbr_settings.inner = app_settings.pbr_settings;
        self.pbr_settings.update();

        // TODO: let this be user-configurable
        let persp = Mat4::perspective_rh_gl(
            f32::to_radians(60.),
            app_settings.viewport_dim.width / app_settings.viewport_dim.height,
            0.1,
            1000.,
        );

        let scene = res.get_scene(selected_scene)?;
        self.transforms.inner.projection = persp;
        self.transforms.inner.view = camera.view_mat();
        self.transforms.inner.model = scene.transform;
        self.transforms.update();

        self.lighting.inner.cam_pos = camera.get_pos().extend(0.0);
        self.lighting.update();

        Ok(())
    }

    fn update_brdf(&mut self, res: &mut RefMut<Resources>) -> Result<()> {
        let material_src = self.app_settings.get().material_src;
        match material_src {
            MaterialSrc::MerlBrdf => {
                let selected_brdf = self.app_settings.get().selected_merl_brdf;
                let brdf = res.get_merl_brdf(selected_brdf)?;
                self.check_load_brdf(brdf)?;
            }
            MaterialSrc::MitBrdf => {
                let selected_brdf = self.app_settings.get().selected_mit_brdf;
                let brdf = res.get_mit_brdf(selected_brdf)?;
                self.check_load_brdf(brdf)?;
            }
            MaterialSrc::UtiaBrdf => {
                let selected_brdf = self.app_settings.get().selected_utia_brdf;
                let brdf = res.get_utia_brdf(selected_brdf)?;
                self.check_load_brdf(brdf)?;
            }
            _ => (),
        };

        Ok(())
    }

    fn check_load_brdf<const BINDING: u32>(&mut self, brdf: &mut BrdfRaw<BINDING>) -> Result<()> {
        if brdf.ibl_texture.is_none() {
            let ibl_texture = Ibl::compute_ibl_brdf(&brdf.ssbo, &self.cubemap, brdf.typ)?;
            brdf.ibl_texture = Some(ibl_texture);
        }

        brdf.ssbo.bind();
        unsafe {
            gl::BindTextureUnit(ogl::RAW_BRDF_PORT, brdf.ibl_texture.as_ref().unwrap().id);
        }

        Ok(())
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

            let app_settings = self.app_settings.get();
            gl::Viewport(
                app_settings.viewport_dim.min_x as i32,
                app_settings.viewport_dim.min_y as i32,
                app_settings.viewport_dim.width as i32,
                app_settings.viewport_dim.height as i32,
            );
            gl::ClearColor(0.15, 0.15, 0.15, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS);

            // TODO(high): enable / disable alpha blending based on GLTF
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    /// Recursive - traverses the node hierarchy and handles each node.
    fn render_node(&mut self, node: &Node, outer_transform: Mat4) -> Result<()> {
        let next_level_transform = outer_transform * node.transform;

        if let Some(mesh) = &node.mesh {
            self.render_mesh(mesh, next_level_transform)?;
        }

        for node in &node.children {
            self.render_node(node, next_level_transform)?;
        }

        Ok(())
    }

    /// Renders the mesh of a node
    fn render_mesh(&mut self, mesh: &Mesh, node_transform: Mat4) -> Result<()> {
        self.transforms.inner.model = node_transform;
        self.transforms.update();

        for primitive in &mesh.primitives {
            self.bind_textures(primitive);
            self.set_material(primitive);

            let shader = match self.app_settings.get().material_src {
                b @ (MaterialSrc::MerlBrdf | MaterialSrc::MitBrdf | MaterialSrc::UtiaBrdf) => {
                    let brdf_typ = match b {
                        MaterialSrc::MerlBrdf => BrdfType::Merl,
                        MaterialSrc::MitBrdf => BrdfType::Mit,
                        MaterialSrc::UtiaBrdf => BrdfType::Utia,
                        _ => unreachable!(),
                    };
                    let defines = DataDrivenDefines::from_prim_brdf(primitive, brdf_typ);
                    self.shaders.data_based_shaders.get_shader(defines)?
                }
                _ => {
                    let defines = PbrDefines::from_prim(primitive);
                    self.shaders.pbr_shaders.get_shader(defines)?
                }
            };

            shader.use_shader(|| {
                Self::draw_mesh(primitive);
            })
        }

        Ok(())
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
            gl::BindTextureUnit(ogl::IRRADIANCE_PORT, self.ibl.textures.irradiance_tex_id.id);
            gl::BindTextureUnit(ogl::PREFILTER_PORT, self.ibl.textures.prefilter_tex_id.id);
            gl::BindTextureUnit(ogl::BRDF_PORT, self.ibl.textures.dfg_lut_id.id);
            gl::BindTextureUnit(ogl::CUBEMAP_PORT, self.cubemap.id);
        }
    }

    fn set_material(&mut self, prim: &Primitive) {
        let app_settings = self.app_settings.get();
        if app_settings.material_src == MaterialSrc::PbrOverride {
            self.material.inner = app_settings.pbr_material_override;
        } else if app_settings.material_src == MaterialSrc::Gltf {
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

    fn render_lights(&mut self) -> Result<()> {
        let lighting = self.lighting.inner;
        let num_lights = lighting.lights;

        let prim = &self.sphere.root.children[0]
            .mesh
            .as_ref()
            .ok_or(eyre!("no mesh in light object"))?
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

        Ok(())
    }
}
