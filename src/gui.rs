use egui::{CtxRef, RichText, Ui};

use crate::{camera::CameraTyp, resources::Resources, util::RcMut, AppSettings};

/// All state that needs to be rendered in the GUI
pub struct GuiCtx {
    pub resources: RcMut<Resources>,
    pub app_settings: RcMut<AppSettings>,
}

/// Implements methods for displaying the widgets.
impl GuiCtx {
    /// Creates the GUI.
    ///
    /// Immediate mode GUI - is called every frame.
    pub fn create_gui(&mut self, egui_ctx: &mut CtxRef) {
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .max_width(400.0)
            .show(egui_ctx, |ui| {
                self.left_panel(ui);
            });

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .max_width(400.0)
            .show(egui_ctx, |ui| {
                self.right_panel(ui);
            });

        let ppp = egui_ctx.pixels_per_point();
        let rect = egui_ctx.available_rect();

        let mut app_settings = self.app_settings.get_mut();
        app_settings.viewport_dim.min_x = ppp * rect.left();
        app_settings.viewport_dim.min_y = ppp * rect.top();
        app_settings.viewport_dim.width = ppp * rect.width();
        app_settings.viewport_dim.height = ppp * rect.height();
    }

    fn right_panel(&mut self, ui: &mut Ui) {
        let mut app_settings = self.app_settings.get_mut();

        egui::global_dark_light_mode_switch(ui);

        ui.group(|ui| {
            ui.add(egui::Label::new(RichText::new("Camera").heading().strong()));
            ui.separator();

            ui.menu_button("Mode", |ui| {
                if ui.button("Orbital").clicked() {
                    app_settings.camera_typ = CameraTyp::Orbital;
                    ui.close_menu();
                }

                if ui.button("Flycam").clicked() {
                    app_settings.camera_typ = CameraTyp::Flycam;
                    ui.close_menu();
                }
            });
        });

        ui.group(|ui| {
            ui.add(egui::Label::new(
                RichText::new("PBR Material").heading().strong(),
            ));
            ui.separator();

            ui.checkbox(
                &mut app_settings.should_override_material,
                "Override material",
            );

            ui.add_enabled_ui(app_settings.should_override_material, |ui| {
                ui.separator();

                ui.color_edit_button_rgba_unmultiplied(
                    &mut app_settings.pbr_material_override.base_color_factor,
                );

                ui.add(
                    egui::Slider::new(
                        &mut app_settings.pbr_material_override.metallic_factor,
                        0.0..=1.0,
                    )
                    .text("Metallic")
                    .smart_aim(false),
                );

                ui.add(
                    egui::Slider::new(
                        &mut app_settings.pbr_material_override.roughness_factor,
                        0.0..=1.0,
                    )
                    .text("Roughness")
                    .smart_aim(false),
                );

                ui.add(
                    egui::Slider::new(
                        &mut app_settings.pbr_material_override.anisotropy,
                        -1.0..=1.0,
                    )
                    .text("Anisotropy")
                    .smart_aim(false),
                );
            });
        });

        ui.group(|ui| {
            ui.add(egui::Label::new(
                RichText::new("PBR Render settings").heading().strong(),
            ));
            ui.separator();

            let mut clearcoat_enabled = app_settings.pbr_settings.clearcoat_enabled();
            let mut direct_light_enabled = app_settings.pbr_settings.direct_light_enabled();
            let mut ibl_enabled = app_settings.pbr_settings.ibl_enabled();

            ui.checkbox(&mut clearcoat_enabled, "Clearcoat enabled");
            ui.checkbox(&mut direct_light_enabled, "Direct light enabled");
            ui.checkbox(&mut ibl_enabled, "IBL enabled");

            app_settings
                .pbr_settings
                .set_clearcoat_enabled(clearcoat_enabled);
            app_settings
                .pbr_settings
                .set_direct_light_enabled(direct_light_enabled);
            app_settings.pbr_settings.set_ibl_enabled(ibl_enabled);
        });

        ui.group(|ui| {
            ui.add(egui::Label::new(
                RichText::new("Data-driven rendering").heading().strong(),
            ));
            ui.separator();

            ui.checkbox(&mut app_settings.data_driven_rendering, "Enabled");
        });
    }

    fn left_panel(&mut self, ui: &mut Ui) {
        let mut resources = self.resources.get_mut();
        let mut app_settings = self.app_settings.get_mut();

        ui.group(|ui| {
            ui.add(egui::Label::new(RichText::new("Scenes").heading().strong()));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, scene) in resources.scenes.iter().enumerate() {
                    if ui.button(scene.name()).clicked() {
                        app_settings.selected_scene = i;
                    }
                }
            });
        });

        ui.group(|ui| {
            ui.add(egui::Label::new(
                RichText::new("BRDF data").heading().strong(),
            ));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, brdf) in resources.brdfs.iter().enumerate() {
                    if ui.button(brdf.name()).clicked() {
                        app_settings.selected_brdf = i;
                    }
                }
            });
        });

        if ui.button("Unload resources").clicked() {
            resources.unload();
        }
    }
}
