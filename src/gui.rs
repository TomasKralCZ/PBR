use egui::{CollapsingHeader, CtxRef, RichText, Slider};
use egui_inspect::EguiInspect;
use glam::Vec3;

use crate::{camera::Camera, model::Model, renderer::Renderer};

/// All state that needs to be rendered in the GUI
pub struct Ctx<'a> {
    pub models: &'a mut [Model],
    pub camera: &'a mut Camera,
    pub renderer: &'a mut Renderer,
}

/// Contains the current state of the GUI.
/// Implements methods for displaying the widgets.
pub struct Gui {
    /// Default 0 (assuming that there is at least 1 model in the scene)
    pub selected_model: usize,
    /// If the mesh should be visible
    pub mesh_visible: bool,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            selected_model: 0,
            mesh_visible: true,
        }
    }

    /// Creates the GUI.
    ///
    /// Immediate mode GUI - is called every frame.
    pub fn create_gui(&mut self, ctx: Ctx, egui_ctx: &mut CtxRef) {
        //self.gui_model_hierarchy_window(scene, egui_ctx);
        self.gui_side_panel(ctx, egui_ctx);
    }

    /// Creates a gui for the side panel
    fn gui_side_panel(&mut self, ctx: Ctx, egui_ctx: &mut CtxRef) {
        egui::SidePanel::right("Side Panel").show(egui_ctx, |ui| {
            ui.group(|ui| {
                ui.add(egui::Label::new(RichText::new("Scenes").heading().strong()));
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, model) in ctx.models.iter().enumerate() {
                        if ui.button(&model.name).clicked() {
                            self.selected_model = i;
                        }
                    }
                });
            });

            ui.group(|ui| {
                ui.add(egui::Label::new(
                    RichText::new("Settings").heading().strong(),
                ));

                ui.separator();

                ui.add(
                    Slider::new(&mut ctx.camera.move_speed, 0.0..=0.2)
                        .text("Camera move speed")
                        .smart_aim(false),
                );

                if ui.button("Reset Camera").clicked() {
                    ctx.camera.set_pos(Vec3::new(0.0, 0.0, 3.0));
                }

                egui::global_dark_light_mode_switch(ui);
            });

            ui.group(|ui| {
                ctx.renderer.material.inner.inspect_mut("Material", ui);
            });

            ui.group(|ui| {
                CollapsingHeader::new("Shaders")
                    .selectable(true)
                    .show(ui, |ui| {
                        ctx.renderer.shaders.inspect_mut("Shaders", ui);
                    });
            })
        });
    }
}
