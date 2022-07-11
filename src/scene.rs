use glam::Vec3;

use eyre::Result;

use crate::{model::Model, util::timed_scope};

pub struct Scene {
    pub models: Vec<Model>,
}

impl Scene {
    /// Adds models to the scene
    pub fn init() -> Result<Self> {
        let mut models = Vec::new();

        let mut add = |path: &str| -> Result<()> {
            timed_scope(&format!("Loading '{path}'"), || {
                let model = Model::from_gltf(path)?;
                models.push(model);
                Ok(())
            })
        };
        add("resources/sketchfab_pbr_material_reference_chart/materials.gltf")?;

        add("resources/helmet/DamagedHelmet.gltf")?;

        add("resources/MetalRoughSpheres/glTF-Binary/MetalRoughSpheres.glb")?;

        add("resources/Sphere.glb")?;

        add("resources/bottle/WaterBottle.gltf")?;
        models.last_mut().unwrap().transform = glam::Mat4::from_scale(Vec3::splat(20.0));

        Ok(Self { models })
    }
}
