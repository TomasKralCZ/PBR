use crate::ogl::shader::Shader;
use egui_inspect::EguiInspect;
use eyre::Result;

#[derive(EguiInspect)]
pub struct Shaders {
    /// Shader for meshes containing texture data
    #[inspect(custom_func_mut = "shader_inspect")]
    pub texture_shader: Shader,
    /// Shader for sphere demonstration
    #[inspect(custom_func_mut = "shader_inspect")]
    pub sphere_shader: Shader,
    /// Shader for drawing lights
    #[inspect(custom_func_mut = "shader_inspect")]
    pub light_shader: Shader,
}

impl Shaders {
    pub fn new() -> Result<Self> {
        let texture_shader = Shader::with_file("shaders/basic.vert", "shaders/texture.frag")?;
        let sphere_shader = Shader::with_file("shaders/basic.vert", "shaders/sphere.frag")?;
        let light_shader = Shader::with_file("shaders/basic.vert", "shaders/light.frag")?;

        Ok(Self {
            texture_shader,
            sphere_shader,
            light_shader,
        })
    }
}

fn shader_inspect(shader: &mut Shader, label: &'static str, ui: &mut egui::Ui) {
    if ui.button("Reload").clicked() {
        if let Ok(new_shader) = shader.reload() {
            *shader = new_shader;
        }
    }

    shader.inspect(label, ui);
}
