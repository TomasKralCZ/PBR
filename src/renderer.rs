use eyre::Result;
use glam::{Mat4, Vec3};

use crate::{
    app::AppState,
    camera::Camera,
    gui::RenderViewportDim,
    model::{Mesh, Model, Node, Primitive},
    ogl::{self, uniform_buffer::UniformBuffer},
};

mod hdri;
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
    cube_vao: u32,
    cubemap_tex_id: u32,
    irradiance_map_id: u32,
}

impl Renderer {
    /// Create a new renderer
    pub fn new() -> Result<Self> {
        let cube_vao = Self::init_cube();

        let (cubemap_tex_id, irradiance_map_id) = hdri::load_cubemaps(cube_vao)?;

        Ok(Self {
            shaders: Shaders::new()?,
            transforms: UniformBuffer::new(Transforms::new_indentity()),
            material: UniformBuffer::new(Material::new()),
            lighting: UniformBuffer::new(Lighting::new()),
            sphere: Model::from_gltf("resources/Sphere.glb")?,
            cube_vao,
            cubemap_tex_id,
            irradiance_map_id,
        })
    }

    /// Render a new frame
    pub fn render(&mut self, model: &mut Model, camera: &mut dyn Camera, appstate: &AppState) {
        Self::reset_gl_state(&appstate.render_viewport_dim);

        let persp = Mat4::perspective_rh_gl(
            f32::to_radians(60.),
            appstate.render_viewport_dim.width / appstate.render_viewport_dim.height,
            0.1,
            100.,
        );

        self.transforms.inner.projection = persp;
        self.transforms.inner.view = camera.view_mat();
        self.transforms.inner.model = model.transform;
        self.transforms.update();

        self.lighting.inner.cam_pos = camera.get_pos().extend(0.0);
        self.lighting.update();

        self.render_lights();

        let transform = model.transform;
        self.render_node(&mut model.root, transform, appstate);

        self.shaders.cubemap_shader.draw_with(|| unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.cubemap_tex_id);

            Self::draw_cube(self.cube_vao);
        });
    }

    fn reset_gl_state(viewport_dim: &RenderViewportDim) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);
            gl::Enable(gl::MULTISAMPLE);
            gl::Viewport(
                viewport_dim.min_x as i32,
                viewport_dim.min_y as i32,
                viewport_dim.width as i32,
                viewport_dim.height as i32,
            );
            gl::ClearColor(0.15, 0.15, 0.15, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    /// Recursive - traverses the node hierarchy and handles each node.
    fn render_node(&mut self, node: &mut Node, outer_transform: Mat4, appstate: &AppState) {
        let next_level_transform = outer_transform * node.transform;

        if let Some(mesh) = &node.mesh {
            self.render_mesh(mesh, next_level_transform, appstate);
        }

        for node in &mut node.children {
            self.render_node(node, next_level_transform, appstate);
        }
    }

    /// Renders the mesh of a node
    fn render_mesh(&mut self, mesh: &Mesh, node_transform: Mat4, appstate: &AppState) {
        self.transforms.inner.model = node_transform;
        self.transforms.update();

        for prim in &mesh.primitives {
            unsafe {
                let set_texture = |id: Option<u32>, port: u32| {
                    if let Some(tex_id) = id {
                        gl::BindTextureUnit(port, tex_id);
                    }
                };

                set_texture(prim.base_color_texture, ogl::ALBEDO_PORT);
                set_texture(prim.mr_texture, ogl::MR_PORT);
                set_texture(prim.normal_texture, ogl::NORMAL_PORT);
                set_texture(prim.occlusion_texture, ogl::OCCLUSION_PORT);
                set_texture(prim.emissive_texture, ogl::EMISSIVE_PORT);

                gl::ActiveTexture(gl::TEXTURE0 + ogl::IRRADIANCE_PORT);
                gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.irradiance_map_id);

                self.set_material(prim, appstate);

                let shader = match (
                    prim.base_color_texture,
                    prim.mr_texture,
                    prim.normal_texture,
                    prim.occlusion_texture,
                    prim.emissive_texture,
                ) {
                    (None, None, None, None, None) => &self.shaders.sphere_shader,
                    (Some(_), Some(_), None, None, None) => &self.shaders.pbr,
                    (Some(_), Some(_), None, None, Some(_)) => &self.shaders.pbr_emissive,
                    (Some(_), Some(_), None, Some(_), None) => &self.shaders.pbr_occlusion,
                    (Some(_), Some(_), None, Some(_), Some(_)) => {
                        &self.shaders.pbr_occlusion_emissive
                    }
                    (Some(_), Some(_), Some(_), None, None) => &self.shaders.pbr_normal,
                    (Some(_), Some(_), Some(_), None, Some(_)) => &self.shaders.pbr_normal_emissive,
                    (Some(_), Some(_), Some(_), Some(_), None) => {
                        &self.shaders.pbr_normal_occlusion
                    }
                    (Some(_), Some(_), Some(_), Some(_), Some(_)) => {
                        &self.shaders.pbr_normal_occlusion_emissive
                    }
                    _ => panic!(
                        "Missing a basic texture: {:?}",
                        (
                            prim.base_color_texture,
                            prim.mr_texture,
                            prim.normal_texture,
                            prim.occlusion_texture,
                            prim.emissive_texture
                        )
                    ),
                };

                shader.draw_with(|| {
                    Self::draw_mesh(prim);
                })
            }
        }
    }

    fn set_material(&mut self, prim: &Primitive, appstate: &AppState) {
        if appstate.should_override_material {
            self.material.inner = appstate.pbr_material_override;
        } else {
            self.material.inner.base_color_factor = prim.base_color_factor;
            self.material.inner.emissive_factor[0..3].copy_from_slice(&prim.emissive_factor);
            self.material.inner.metallic_factor = prim.metallic_factor;
            self.material.inner.roughness_factor = prim.roughness_factor;
            self.material.inner.normal_scale = prim.normal_scale;
            self.material.inner.occlusion_strength = prim.occlusion_strength;
        }

        self.material.update();
    }

    fn draw_mesh(prim: &Primitive) {
        unsafe {
            gl::BindVertexArray(prim.vao);

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

    fn init_cube() -> u32 {
        let vertices: [CubeVertex; 8] = [
            CubeVertex::new([-1., -1., 1.]),
            CubeVertex::new([1., -1., 1.]),
            CubeVertex::new([1., 1., 1.]),
            CubeVertex::new([-1., 1., 1.]),
            CubeVertex::new([-1., -1., -1.]),
            CubeVertex::new([1., -1., -1.]),
            CubeVertex::new([1., 1., -1.]),
            CubeVertex::new([-1., 1., -1.]),
        ];

        #[rustfmt::skip]
        let indices: [u8; 36] = [
            0, 2, 1,
            0, 3, 2,
            1, 6, 5,
            1, 2, 6,
            5, 7, 4,
            5, 6, 7,
            4, 3, 0,
            4, 7, 3,
            3, 7, 6,
            3, 6, 2,
            4, 0, 1,
            4, 1, 5,
        ];

        let mut vao = 0;
        let mut ibo = 0;

        unsafe {
            gl::CreateVertexArrays(1, &mut vao);
            ogl::attach_float_buf(vao, &vertices, 3, ogl::POSITION_INDEX, gl::FLOAT);

            gl::CreateBuffers(1, &mut ibo);
            gl::NamedBufferData(
                ibo,
                indices.len() as isize,
                indices.as_ptr() as _,
                gl::STATIC_DRAW,
            );
            gl::VertexArrayElementBuffer(vao, ibo);
        }

        vao
    }

    fn draw_cube(cube_vao: u32) {
        unsafe {
            gl::BindVertexArray(cube_vao);
            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_BYTE, 0 as _);
            gl::BindVertexArray(0);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CubeVertex {
    pos: [f32; 3],
}

impl CubeVertex {
    fn new(pos: [f32; 3]) -> Self {
        Self { pos }
    }
}
