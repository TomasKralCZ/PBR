use eyre::Result;
use glam::Vec3;

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

        add("resources/helmet/DamagedHelmet.gltf")?;

        add("resources/MetalRoughSpheres/glTF-Binary/MetalRoughSpheres.glb")?;

        add("resources/Sphere.glb")?;

        add("resources/kasatka_71m_-_three-bolt_equipment/kasatka.gltf")?;

        add("resources/bottle/WaterBottle.gltf")?;

        add("resources/ToyCar.glb")?;

        add("resources/santa_conga_freebiexmass/drum.gltf")?;

        add("resources/game_ready_pbr_microscope/microscope.gltf")?;

        let len = models.len();
        models[len - 1].transform = glam::Mat4::from_scale(Vec3::splat(0.05));

        models[len - 2].transform = glam::Mat4::from_scale(Vec3::splat(3.0));

        models[len - 3].transform = glam::Mat4::from_scale(Vec3::splat(20.0));

        Ok(Self { models })
    }
}
