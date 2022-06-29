use std::ptr;

use eyre::Result;
use glam::{Mat4, Vec3};

use crate::{
    camera::Camera,
    gui::Gui,
    model::{Mesh, Model, Node, Primitive},
    ogl::{self, uniform_buffer::UniformBuffer},
    scene::Scene,
    window::MyWindow,
};

mod lighting;
pub mod material;
mod shaders;
mod transforms;

pub use material::Material;

use self::{lighting::Lighting, shaders::Shaders, transforms::Transforms};

/// A component responsible for rendering the scene.
pub struct Renderer {
    pub shaders: Shaders,
    /// Current MVP transformation matrices
    transforms: UniformBuffer<Transforms>,
    /// Current mesh material
    pub material: UniformBuffer<Material>,
    /// Current lighting settings
    lighting: UniformBuffer<Lighting>,
    sphere: Model,
}

impl Renderer {
    /// Create a new renderer
    pub fn new() -> Result<Self> {
        Ok(Self {
            shaders: Shaders::new()?,
            transforms: UniformBuffer::new(Transforms::new_indentity()),
            material: UniformBuffer::new(Material::new()),
            lighting: UniformBuffer::new(Lighting::new()),
            sphere: Model::from_gltf("resources/Sphere.glb")?,
        })
    }

    /// Render a new frame
    pub fn render(
        &mut self,
        scene: &mut Scene,
        camera: &mut Camera,
        window: &MyWindow,
        gui_state: &Gui,
    ) {
        Self::reset_gl_state(window);

        // TODO: možná glu perspective
        let persp = Mat4::perspective_rh(
            f32::to_radians(60.),
            window.width as f32 / window.height as f32,
            0.1,
            100.,
        );

        let model = &mut scene.models[gui_state.selected_model];

        self.transforms.inner.projection = persp;
        self.transforms.inner.view = camera.view_mat();
        self.transforms.inner.model = model.transform;
        self.transforms.update();

        self.lighting.inner.cam_pos = camera.get_pos().extend(0.0);
        self.lighting.update();

        self.material.update();

        self.render_lights();

        let transform = model.transform;
        self.render_node(&mut model.root, transform);
    }

    fn reset_gl_state(window: &MyWindow) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);
            gl::Enable(gl::MULTISAMPLE);
            gl::Viewport(0, 0, window.width as i32, window.height as i32);
            gl::ClearColor(0.15, 0.15, 0.15, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    /// Recursive - traverses the node hierarchy and handles each node.
    fn render_node(&mut self, node: &mut Node, outer_transform: Mat4) {
        let next_level_transform = outer_transform * node.transform;

        if let Some(mesh) = &node.mesh {
            self.render_mesh(mesh, next_level_transform);
        }

        for node in &mut node.children {
            self.render_node(node, next_level_transform);
        }
    }

    /// Renders the mesh of a node
    fn render_mesh(&mut self, mesh: &Mesh, node_transform: Mat4) {
        self.transforms.inner.model = node_transform;
        self.transforms.update();

        for prim in &mesh.primitives {
            unsafe {
                let set_texture = |id: Option<u32>, port: u32| {
                    if let Some(tex_id) = id {
                        gl::ActiveTexture(gl::TEXTURE0 + port);
                        gl::BindTexture(gl::TEXTURE_2D, tex_id);
                    }
                };

                set_texture(prim.albedo_texture, ogl::ALBEDO_PORT);
                set_texture(prim.mr_texture, ogl::MR_PORT);
                set_texture(prim.normal_texture, ogl::NORMAL_PORT);
                set_texture(prim.occlusion_texture, ogl::OCCLUSION_PORT);
                set_texture(prim.emissive_texture, ogl::EMISSIVE_PORT);

                match (
                    prim.albedo_texture,
                    prim.mr_texture,
                    prim.normal_texture,
                    prim.occlusion_texture,
                    prim.emissive_texture,
                ) {
                    (None, None, None, None, None) => self.shaders.sphere_shader.draw_with(|| {
                        Self::draw_mesh(prim);
                    }),
                    (Some(_), Some(_), None, None, None) => todo!(),
                    (Some(_), Some(_), None, None, Some(_)) => todo!(),
                    (Some(_), Some(_), None, Some(_), None) => todo!(),
                    (Some(_), Some(_), None, Some(_), Some(_)) => todo!(),
                    (Some(_), Some(_), Some(_), None, None) => todo!(),
                    (Some(_), Some(_), Some(_), None, Some(_)) => todo!(),
                    (Some(_), Some(_), Some(_), Some(_), None) => todo!(),
                    (Some(_), Some(_), Some(_), Some(_), Some(_)) => {
                        self.shaders.texture_shader.draw_with(|| {
                            Self::draw_mesh(prim);
                        })
                    }
                    _ => panic!(
                        "Missing a basic texture: {:?}",
                        (
                            prim.albedo_texture,
                            prim.mr_texture,
                            prim.normal_texture,
                            prim.occlusion_texture,
                            prim.emissive_texture
                        )
                    ),
                }
            }
        }
    }

    fn draw_mesh(prim: &Primitive) {
        unsafe {
            gl::BindVertexArray(prim.vao);

            gl::DrawElements(
                gl::TRIANGLES,
                prim.indices.len() as i32,
                prim.indices.gl_type(),
                ptr::null(),
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
            self.shaders.light_shader.draw_with(|| {
                self.transforms.inner.model = Mat4::from_translation(light_pos.truncate())
                    * Mat4::from_scale(Vec3::splat(0.1));
                self.transforms.update();

                self.shaders
                    .light_shader
                    .set_vec3(light_color.truncate(), "lightColor\0");

                Self::draw_mesh(prim);
            });
        }
    }
}
