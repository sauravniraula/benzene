use benzene::{Entity, Scene};
use egui::DragValue;
use nalgebra::{Vector3, Vector4};

pub(super) fn draw_debug_ui(context: &egui::Context, dt: f32, scene: &mut Scene) -> egui::Rect {
    let frame_ms = dt * 1_000.0;
    let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };

    egui::TopBottomPanel::bottom("bottom_status_bar")
        .exact_height(72.0)
        .show(context, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("benzene");
                ui.separator();
                ui.label(format!("{frame_ms:.2} ms"));
                ui.label(format!("{fps:.0} fps"));
                ui.separator();
                ui.label("WASD: move");
                ui.label("Left mouse: capture/look");
                ui.separator();
                ui.label("egui panels rendered by Ash");
            });
        });

    egui::SidePanel::left("scene_sidebar")
        .default_width(260.0)
        .show(context, |ui| {
            ui.heading("benzene");
            ui.label("Scene");
            ui.separator();
            ui.label(format!("active camera: {:?}", scene.active_camera()));
            ui.separator();
            ui.strong("Controls");
            ui.label("WASD: move camera");
            ui.label("Space / Left Alt: vertical");
            ui.label("Left mouse: capture and look");
        });

    egui::SidePanel::right("renderer_sidebar")
        .default_width(340.0)
        .show(context, |ui| {
            ui.heading("Entities");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                draw_entity_inspector(ui, scene);
            });
        });

    context.available_rect()
}

fn draw_entity_inspector(ui: &mut egui::Ui, scene: &mut Scene) {
    let entities = scene.entities().collect::<Vec<_>>();
    for entity in entities {
        let label = scene
            .name(entity)
            .map(|name| format!("{}  {entity:?}", name.value))
            .unwrap_or_else(|| format!("{entity:?}"));

        egui::CollapsingHeader::new(label)
            .id_salt(entity)
            .show(ui, |ui| {
                draw_entity_components(ui, scene, entity);
            });
    }
}

fn draw_entity_components(ui: &mut egui::Ui, scene: &mut Scene, entity: Entity) {
    let mut shown_components = 0usize;

    if let Some(visibility) = scene.visibility_mut(entity) {
        shown_components += 1;
        ui.collapsing("Visibility", |ui| {
            ui.checkbox(&mut visibility.visible, "Visible");
        });
    }

    if let Some(name) = scene.name_mut(entity) {
        shown_components += 1;
        ui.collapsing("Name", |ui| {
            ui.text_edit_singleline(&mut name.value);
        });
    }

    if let Some(transform) = scene.transform_mut(entity) {
        shown_components += 1;
        ui.collapsing("Transform", |ui| {
            let mut changed = false;
            ui.label("Position");
            changed |= vector3_editor(ui, &mut transform.position, 0.05);
            ui.separator();

            ui.label("Rotation");
            changed |= rotation_editor(ui, &mut transform.rotation);
            ui.separator();

            ui.label("Scale");
            changed |= vector3_editor(ui, &mut transform.scale, 0.02);

            if changed {
                transform.dirty = true;
            }
        });
    }

    if scene.camera(entity).is_some() {
        shown_components += 1;
        ui.collapsing("Camera", |ui| {
            if scene.active_camera() == Some(entity) {
                ui.label("active camera");
            } else if ui.button("Set active camera").clicked() {
                scene.set_active_camera(entity);
            }

            if let Some(camera) = scene.camera_mut(entity) {
                ui.horizontal(|ui| {
                    ui.label("speed");
                    ui.add(
                        DragValue::new(&mut camera.speed)
                            .speed(0.05)
                            .range(0.0..=100.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("look speed");
                    ui.add(
                        DragValue::new(&mut camera.rotation_speed)
                            .speed(0.005)
                            .range(0.0..=10.0),
                    );
                });
                ui.separator();
                ui.label("Projection");
                let mut fov_degrees = camera.fov_y_radians.to_degrees();
                if ui
                    .horizontal(|ui| {
                        ui.label("vertical fov");
                        ui.add(
                            DragValue::new(&mut fov_degrees)
                                .speed(0.25)
                                .range(1.0..=170.0)
                                .suffix(" deg"),
                        )
                        .changed()
                    })
                    .inner
                {
                    camera.fov_y_radians = fov_degrees.to_radians();
                }
                ui.horizontal(|ui| {
                    ui.label("near clip");
                    ui.add(
                        DragValue::new(&mut camera.near_clip)
                            .speed(0.01)
                            .range(0.001..=100_000.0),
                    );
                });
                camera.near_clip = camera.near_clip.max(0.001);
                ui.horizontal(|ui| {
                    ui.label("far clip");
                    ui.add(
                        DragValue::new(&mut camera.far_clip)
                            .speed(1.0)
                            .range((camera.near_clip + 0.001)..=1_000_000.0),
                    );
                });
                camera.far_clip = camera.far_clip.max(camera.near_clip + 0.001);
            }
        });
    }

    if let Some(light) = scene.point_light_mut(entity) {
        shown_components += 1;
        ui.collapsing("Point Light", |ui| {
            light_editor(ui, &mut light.color, "intensity", 0.0..=200.0);
        });
    }

    if let Some(light) = scene.directional_light_mut(entity) {
        shown_components += 1;
        ui.collapsing("Directional Light", |ui| {
            light_editor(ui, &mut light.color, "strength", 0.0..=10.0);
        });
    }

    if let Some(light) = scene.spot_light_mut(entity) {
        shown_components += 1;
        ui.collapsing("Spot Light", |ui| {
            light_editor(ui, &mut light.color, "intensity", 0.0..=200.0);
        });
    }

    if let Some(mesh_instance) = scene.mesh_instance(entity) {
        shown_components += 1;
        ui.collapsing("Mesh Instance", |ui| {
            ui.label(format!("mesh: {:?}", mesh_instance.mesh));
            ui.label(format!("material: {:?}", mesh_instance.material));
            ui.label("Asset references are read-only until the asset picker exists.");
        });
    }

    if shown_components == 0 {
        ui.label("No components");
    }
}

fn vector3_editor(ui: &mut egui::Ui, value: &mut Vector3<f32>, speed: f64) -> bool {
    let mut changed = false;
    changed |= drag_f32(ui, "x", &mut value.x, speed);
    changed |= drag_f32(ui, "y", &mut value.y, speed);
    changed |= drag_f32(ui, "z", &mut value.z, speed);
    changed
}

fn rotation_editor(ui: &mut egui::Ui, value: &mut Vector3<f32>) -> bool {
    let mut changed = false;
    changed |= drag_degrees(ui, "pitch", &mut value.x);
    changed |= drag_degrees(ui, "yaw", &mut value.y);
    changed |= drag_degrees(ui, "roll", &mut value.z);
    changed
}

fn light_editor(
    ui: &mut egui::Ui,
    color: &mut Vector4<f32>,
    intensity_label: &str,
    intensity_range: std::ops::RangeInclusive<f32>,
) {
    ui.horizontal(|ui| {
        ui.label("r");
        ui.add(DragValue::new(&mut color.x).speed(0.01).range(0.0..=1.0));
    });
    ui.horizontal(|ui| {
        ui.label("g");
        ui.add(DragValue::new(&mut color.y).speed(0.01).range(0.0..=1.0));
    });
    ui.horizontal(|ui| {
        ui.label("b");
        ui.add(DragValue::new(&mut color.z).speed(0.01).range(0.0..=1.0));
    });
    ui.horizontal(|ui| {
        ui.label(intensity_label);
        ui.add(
            DragValue::new(&mut color.w)
                .speed(0.05)
                .range(intensity_range),
        );
    });
}

fn drag_f32(ui: &mut egui::Ui, label: &str, value: &mut f32, speed: f64) -> bool {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(DragValue::new(value).speed(speed)).changed()
    })
    .inner
}

fn drag_degrees(ui: &mut egui::Ui, label: &str, radians: &mut f32) -> bool {
    let mut degrees = radians.to_degrees();
    let changed = ui
        .horizontal(|ui| {
            ui.label(label);
            ui.add(DragValue::new(&mut degrees).speed(0.25).suffix(" deg"))
                .changed()
        })
        .inner;
    if changed {
        *radians = degrees.to_radians();
    }
    changed
}
