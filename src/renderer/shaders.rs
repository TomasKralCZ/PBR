use crate::ogl::shader::Shader;
//use egui_inspect::EguiInspect;
use eyre::Result;

//#[derive(EguiInspect)]
pub struct Shaders {
    /// Shader for meshes containing texture data
    //#[inspect(custom_func_mut = "shader_inspect")]
    pub pbr_normal_occlusion_emissive: Shader,
    pub pbr_normal_occlusion: Shader,
    pub pbr_normal_emissive: Shader,
    pub pbr_normal: Shader,
    pub pbr_occlusion_emissive: Shader,
    pub pbr_occlusion: Shader,
    pub pbr_emissive: Shader,
    pub pbr: Shader,

    /// Shader for sphere demonstration
    //#[inspect(custom_func_mut = "shader_inspect")]
    pub sphere_shader: Shader,
    /// Shader for drawing lights
    //#[inspect(custom_func_mut = "shader_inspect")]
    pub light_shader: Shader,

    pub cubemap_shader: Shader,
}

impl Shaders {
    pub fn new() -> Result<Self> {
        let pbr_normal_occlusion_emissive = Shader::with_files_defines(
            "shaders/basic.vert",
            &[],
            "shaders/pbr.frag",
            &["NORMAL_MAP", "OCCLUSION_MAP", "EMISSIVE_MAP"],
        )?;

        let pbr_normal_occlusion = Shader::with_files_defines(
            "shaders/basic.vert",
            &[],
            "shaders/pbr.frag",
            &["NORMAL_MAP", "OCCLUSION_MAP"],
        )?;

        let pbr_normal_emissive = Shader::with_files_defines(
            "shaders/basic.vert",
            &[],
            "shaders/pbr.frag",
            &["NORMAL_MAP", "EMISSIVE_MAP"],
        )?;

        let pbr_normal = Shader::with_files_defines(
            "shaders/basic.vert",
            &[],
            "shaders/pbr.frag",
            &["NORMAL_MAP"],
        )?;

        let pbr_occlusion_emissive = Shader::with_files_defines(
            "shaders/basic.vert",
            &[],
            "shaders/pbr.frag",
            &["OCCLUSION_MAP", "EMISSIVE_MAP"],
        )?;

        let pbr_occlusion = Shader::with_files_defines(
            "shaders/basic.vert",
            &[],
            "shaders/pbr.frag",
            &["OCCLUSION_MAP"],
        )?;

        let pbr_emissive = Shader::with_files_defines(
            "shaders/basic.vert",
            &[],
            "shaders/pbr.frag",
            &["EMISSIVE_MAP"],
        )?;

        let pbr = Shader::with_files_defines("shaders/basic.vert", &[], "shaders/pbr.frag", &[])?;

        let sphere_shader = Shader::with_files("shaders/basic.vert", "shaders/sphere.frag")?;

        let light_shader = Shader::with_files("shaders/basic.vert", "shaders/light.frag")?;

        let cubemap_shader = Shader::with_files("shaders/cubemap.vert", "shaders/cubemap.frag")?;

        Ok(Self {
            pbr_normal_occlusion_emissive,
            sphere_shader,
            light_shader,
            pbr_normal_occlusion,
            pbr_normal_emissive,
            pbr_normal,
            pbr_occlusion_emissive,
            pbr_occlusion,
            pbr_emissive,
            pbr,
            cubemap_shader,
        })
    }
}

/* fn shader_inspect(shader: &mut Shader, label: &'static str, ui: &mut egui::Ui) {
    if ui.button("Reload").clicked() {
        if let Ok(new_shader) = shader.reload() {
            *shader = new_shader;
        }
    }

    shader.inspect(label, ui);
}
 */
