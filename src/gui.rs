use egui::{CtxRef, RichText, Ui};

use crate::{
    camera::{Camera, CameraTyp},
    renderer::Renderer,
    scenes::Scenes,
    AppState,
};

/// All state that needs to be rendered in the GUI
pub struct GuiCtx<'a> {
    pub scenes: &'a mut Scenes,
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

        let ppp = egui_ctx.pixels_per_point();
        let rect = egui_ctx.available_rect();
        self.render_viewport_dim.min_x = ppp * rect.left();
        self.render_viewport_dim.min_y = ppp * rect.top();
        self.render_viewport_dim.width = ppp * rect.width();
        self.render_viewport_dim.height = ppp * rect.height();
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
            ui.add(egui::Label::new(
                RichText::new("Material").heading().strong(),
            ));
            ui.separator();

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

                ui.add(
                    egui::Slider::new(&mut self.pbr_material_override.anisotropy, -1.0..=1.0)
                        .text("Anisotropy")
                        .smart_aim(false),
                );
            });
        });

        ui.group(|ui| {
            ui.add(egui::Label::new(
                RichText::new("Render settings").heading().strong(),
            ));
            ui.separator();

            let mut clearcoat_enabled = gui_ctx.renderer.settings.inner.clearcoat_enabled();
            let mut direct_light_enabled = gui_ctx.renderer.settings.inner.direct_light_enabled();
            let mut ibl_enabled = gui_ctx.renderer.settings.inner.ibl_enabled();

            ui.checkbox(&mut clearcoat_enabled, "Clearcoat enabled");
            ui.checkbox(&mut direct_light_enabled, "Direct light enabled");
            ui.checkbox(&mut ibl_enabled, "IBL enabled");

            gui_ctx
                .renderer
                .settings
                .inner
                .set_clearcoat_enabled(clearcoat_enabled);
            gui_ctx
                .renderer
                .settings
                .inner
                .set_direct_light_enabled(direct_light_enabled);
            gui_ctx.renderer.settings.inner.set_ibl_enabled(ibl_enabled);
        });
    }

    fn left_panel(&mut self, ui: &mut Ui, gui_ctx: &mut GuiCtx) {
        ui.group(|ui| {
            ui.add(egui::Label::new(RichText::new("Models").heading().strong()));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, scene) in gui_ctx.scenes.get_scenes().iter().enumerate() {
                    if ui.button(scene.name()).clicked() {
                        self.selected_scene = Some(i);
                    }
                }
            });
        });
    }
}
