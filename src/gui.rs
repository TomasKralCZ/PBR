use egui::{CtxRef, RichText, Ui};

use crate::{
    app_settings::{self, MaterialSrc},
    camera::CameraTyp,
    resources::Resources,
    util::RcMut,
    AppSettings,
};

/// All state that needs to be rendered in the GUI
pub struct Gui {
    pub resources: RcMut<Resources>,
    pub app_settings: RcMut<AppSettings>,
}

/// Implements methods for displaying the widgets.
impl Gui {
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

            ui.add(egui::Label::new(RichText::new("Material source").strong()));
            ui.separator();
            ui.vertical(|ui| {
                use MaterialSrc::*;
                let selected = &mut app_settings.material_src;

                ui.radio_value(selected, Gltf, Gltf.to_str());
                ui.radio_value(selected, PbrOverride, PbrOverride.to_str());
                ui.radio_value(selected, MerlBrdf, MerlBrdf.to_str());
                ui.radio_value(selected, UtiaBrdf, UtiaBrdf.to_str());
            });

            ui.separator();

            Self::right_panel_material_override(ui, &mut app_settings);
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

            ui.group(|ui| {
                ui.label("Diffuse BRDF");
                ui.separator();

                use app_settings::DiffuseType::*;
                ui.radio_value(
                    &mut app_settings.pbr_settings.diffuse_type,
                    Lambert,
                    Lambert.to_str(),
                );
                ui.radio_value(
                    &mut app_settings.pbr_settings.diffuse_type,
                    Frostbite,
                    Frostbite.to_str(),
                );
                ui.radio_value(
                    &mut app_settings.pbr_settings.diffuse_type,
                    CodWWII,
                    CodWWII.to_str(),
                );
            });
        });
    }

    fn right_panel_material_override(
        ui: &mut Ui,
        app_settings: &mut std::cell::RefMut<AppSettings>,
    ) {
        ui.add_enabled_ui(
            app_settings.material_src == MaterialSrc::PbrOverride,
            |ui| {
                ui.add(egui::Label::new(
                    RichText::new("Material override").strong(),
                ));
                ui.separator();

                let color = &mut app_settings.pbr_material_override.base_color_factor;

                ui.color_edit_button_rgba_unmultiplied(color);

                ui.horizontal(|ui| {
                    let mut color = app_settings.pbr_material_override.base_color_factor;
                    color = color.map(|f| f * 255.);

                    use egui::DragValue;
                    ui.add(
                        DragValue::new(&mut color[0])
                            .prefix("r: ")
                            .clamp_range(0.0..=255.0),
                    );
                    ui.add(
                        DragValue::new(&mut color[1])
                            .prefix("g: ")
                            .clamp_range(0.0..=255.0),
                    );
                    ui.add(
                        DragValue::new(&mut color[2])
                            .prefix("b: ")
                            .clamp_range(0.0..=255.0),
                    );

                    app_settings.pbr_material_override.base_color_factor = color.map(|f| f / 255.);
                });

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
            },
        );
    }

    fn left_panel(&mut self, ui: &mut Ui) {
        let mut resources = self.resources.get_mut();
        let mut app_settings = self.app_settings.get_mut();

        let height = ui.available_height() / 4.;

        ui.group(|ui| {
            ui.add(egui::Label::new(RichText::new("Scenes").heading().strong()));
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(height)
                .id_source("scenes_scroll")
                .show(ui, |ui| {
                    for (i, scene) in resources.scenes.iter().enumerate() {
                        if ui.button(scene.name()).clicked() {
                            app_settings.selected_scene = i;
                        }
                    }
                });
        });

        ui.group(|ui| {
            ui.add(egui::Label::new(
                RichText::new("Environment maps").heading().strong(),
            ));
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(height)
                .id_source("envmaps_scroll")
                .show(ui, |ui| {
                    for (i, envmap) in resources.envmaps.iter().enumerate() {
                        if ui.button(envmap.name()).clicked() {
                            app_settings.selected_envmap = i;
                        }
                    }
                });
        });

        ui.group(|ui| {
            ui.add(egui::Label::new(
                RichText::new("MERL BRDFs").heading().strong(),
            ));
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(height)
                .id_source("merl_scroll")
                .show(ui, |ui| {
                    for (i, brdf) in resources.merl_brdfs.iter().enumerate() {
                        if ui.button(brdf.name()).clicked() {
                            app_settings.selected_merl_brdf = i;
                        }
                    }
                });
        });

        ui.group(|ui| {
            ui.add(egui::Label::new(
                RichText::new("UTIA BRDFs").heading().strong(),
            ));
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(height)
                .id_source("utia_scroll")
                .show(ui, |ui| {
                    for (i, brdf) in resources.utia_brdfs.iter().enumerate() {
                        if ui.button(brdf.name()).clicked() {
                            app_settings.selected_utia_brdf = i;
                        }
                    }
                });
        });

        if ui.button("Unload resources").clicked() {
            resources.unload();
        }
    }
}
