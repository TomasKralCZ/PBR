use egui::{CtxRef, RichText, Ui};

use crate::{
    camera::{Camera, CameraTyp},
    model::Model,
    renderer::Renderer,
    window::AppWindow,
    AppState,
};

/// All state that needs to be rendered in the GUI
pub struct GuiCtx<'a> {
    pub models: &'a mut [Model],
    pub camera: &'a mut dyn Camera,
    pub cam_typ: &'a mut CameraTyp,
    pub renderer: &'a mut Renderer,
}

/// Implements methods for displaying the widgets.
impl AppState {
    /// Creates the GUI.
    ///
    /// Immediate mode GUI - is called every frame.
    pub fn create_gui(&mut self, gui_ctx: &mut GuiCtx, egui_ctx: &mut CtxRef) {
        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .min_height(32.0)
            .show(egui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Expandable Upper Panel");
                    });
                });
            });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .max_width(400.0)
            .show(egui_ctx, |ui| {
                self.left_panel(ui, gui_ctx);
            });

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .max_width(400.0)
            .show(egui_ctx, |ui| {
                self.right_panel(ui, gui_ctx);
            });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .min_height(0.0)
            .show(egui_ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Bottom Panel");
                });
            });

        let ppp = egui_ctx.pixels_per_point();
        let rect = egui_ctx.available_rect();
        self.render_viewport_dim.min_x = ppp * rect.left();
        self.render_viewport_dim.min_y = ppp * rect.top();
        self.render_viewport_dim.width = ppp * rect.width();
        self.render_viewport_dim.height = ppp * rect.height();

        //self.gui_model_hierarchy_window(gui_ctx.models, egui_ctx);
    }

    fn right_panel(&mut self, ui: &mut Ui, gui_ctx: &mut GuiCtx) {
        egui::global_dark_light_mode_switch(ui);

        ui.group(|ui| {
            ui.add(egui::Label::new(RichText::new("Camera").heading().strong()));
            ui.separator();

            ui.menu_button("Mode", |ui| {
                if ui.button("Orbital").clicked() {
                    *gui_ctx.cam_typ = CameraTyp::Orbital;
                    ui.close_menu();
                }

                if ui.button("Flycam").clicked() {
                    *gui_ctx.cam_typ = CameraTyp::Flycam;
                    ui.close_menu();
                }
            });
        });

        ui.group(|ui| {
            ui.checkbox(&mut self.should_override_material, "Override material");

            ui.add_enabled_ui(self.should_override_material, |ui| {
                ui.separator();

                ui.color_edit_button_rgba_unmultiplied(
                    &mut self.pbr_material_override.base_color_factor,
                );

                ui.add(
                    egui::Slider::new(&mut self.pbr_material_override.metallic_factor, 0.0..=1.0)
                        .text("Metallic")
                        .smart_aim(false),
                );

                ui.add(
                    egui::Slider::new(&mut self.pbr_material_override.roughness_factor, 0.0..=1.0)
                        .text("Roughness")
                        .smart_aim(false),
                );
            });
        });
    }

    fn left_panel(&mut self, ui: &mut Ui, gui_ctx: &GuiCtx) {
        ui.group(|ui| {
            ui.add(egui::Label::new(RichText::new("Models").heading().strong()));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, model) in gui_ctx.models.iter().enumerate() {
                    if ui.button(&model.name).clicked() {
                        self.selected_model = Some(i);
                    }
                }
            });
        });
    }

    /* /// Create the subwindow containing the model hierarchy
    fn gui_model_hierarchy_window(&mut self, scene: &mut [Model], egui_ctx: &mut CtxRef) {
        let model = &mut scene[self.selected_model];

        egui::Window::new("Model Hierarchy")
            .scroll2([false, true])
            .resizable(true)
            .show(egui_ctx, |ui| {
                self.gui_node(&mut model.root, ui);
            });
    } */

    /* /// Recursive - creates the node hierarchy inside the model hierarchy window
    fn gui_node(&mut self, node: &mut Node, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if !&node.children.is_empty() {
                CollapsingHeader::new(&node.name)
                    .id_source(node.index)
                    .selectable(true)
                    .show(ui, |ui| {
                        for child_node in &mut node.children {
                            self.gui_node(child_node, ui);
                        }
                    });
            } else {
                ui.label(&node.name);
            }

            if let Some(mesh) = &mut node.mesh {
                self.gui_mesh(mesh, ui);
            }
        });
    } */

    /* fn gui_mesh(&mut self, mesh: &mut Mesh, ui: &mut Ui) {
        if !mesh.primitives.is_empty() {
            CollapsingHeader::new(mesh.name.as_ref().unwrap_or(&"N/A".to_string()))
                .selectable(true)
                .show(ui, |ui| {
                    for prim in &mut mesh.primitives {
                        CollapsingHeader::new(&format!("{}-primitive", prim.vao))
                            .selectable(true)
                            .show(ui, |ui| {
                                prim.inspect_mut("Primitive", ui);
                            });
                    }
                });
        }
    } */
}

pub struct RenderViewportDim {
    pub min_x: f32,
    pub min_y: f32,
    pub width: f32,
    pub height: f32,
}

impl RenderViewportDim {
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
