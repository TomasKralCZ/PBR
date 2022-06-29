use egui_inspect::EguiInspect;

use crate::ogl::uniform_buffer::UniformBufferElement;

/// Uniform buffer element that stores the material settings
#[derive(EguiInspect, Default, bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
#[repr(C)]
pub struct Material {
    #[inspect(custom_func_mut = "arr4_inspect")]
    pub albedo: [f32; 4],
    #[inspect(slider, min = 0., max = 1.0)]
    pub roughness: f32,
    #[inspect(slider, min = 0., max = 1.0)]
    pub metalness: f32,
}

fn arr4_inspect(v: &mut [f32; 4], _: &'static str, ui: &mut egui::Ui) {
    for (ele, label) in v.iter_mut().take(3).zip(['R', 'G', 'B']) {
        ui.add(
            egui::Slider::new(ele, 0.0..=1.)
                .text(label)
                .smart_aim(false),
        );
    }
}

impl Material {
    pub fn new() -> Self {
        Self {
            albedo: [0.31, 0.87, 0.12, 1.0],
            roughness: 1.,
            metalness: 0.,
        }
    }
}

impl UniformBufferElement for Material {
    const BINDING: u32 = 4;
}
