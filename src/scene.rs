use eyre::Result;

use crate::{model::Model, util::timed_scope};

pub struct Scene {
    models: Vec<LazyModel>,
}

impl Scene {
    /// Adds models to the scene
    pub fn init() -> Result<Self> {
        let mut models = Vec::new();

        let mut add = |path: &'static str| {
            let lazy_model = LazyModel::new(path);
            models.push(lazy_model);
        };

        add("resources/helmet/DamagedHelmet.gltf");

        add("resources/MetalRoughSpheres/glTF-Binary/MetalRoughSpheres.glb");

        add("resources/Sphere.glb");

        add("resources/kasatka_71m_-_three-bolt_equipment/kasatka.gltf");

        add("resources/bottle/WaterBottle.gltf");

        add("resources/free_1975_porsche_911_930_turbo/porsche.gltf");

        add("resources/shoe_with_clearcoat/shoe.gltf");

        add("resources/ToyCar.glb");

        add("resources/santa_conga_freebiexmass/drum.gltf");

        add("resources/game_ready_pbr_microscope/microscope.gltf");

        /* let len = models.len();
        models[len - 1].transform = glam::Mat4::from_scale(Vec3::splat(0.05));

        models[len - 2].transform = glam::Mat4::from_scale(Vec3::splat(3.0));

        models[len - 3].transform = glam::Mat4::from_scale(Vec3::splat(20.0)); */

        add("resources/ClearCoatTest.glb");

        Ok(Self { models })
    }

    pub fn get_model(&mut self, index: usize) -> Result<&Model> {
        self.models[index].get()
    }

    pub fn get_models(&self) -> &[LazyModel] {
        &self.models
    }
}

/// Loading models takes a long time, load them lazily
pub struct LazyModel {
    path: &'static str,
    model: Option<Model>,
}

impl LazyModel {
    fn new(path: &'static str) -> Self {
        Self { path, model: None }
    }

    fn get(&mut self) -> Result<&Model> {
        // Can't use if let Some(...) because the borrow checker is angry
        // Can't use get_or_insert_with(...) because I need error handling
        if self.model.is_some() {
            Ok(self.model.as_ref().unwrap())
        } else {
            let path = self.path;

            let model = timed_scope(&format!("Loading '{path}'"), || Model::from_gltf(path))?;

            self.model = Some(model);

            Ok(self.model.as_ref().unwrap())
        }
    }

    pub fn name(&self) -> &str {
        // Find the index where the filename starts (if any)
        let start = self
            .path
            .rfind('/')
            .map(|i| i + 1)
            .unwrap_or(self.path.rfind('\\').map(|i| i + 1).unwrap_or(0));
        // Find the index where the file extension starts (if any)
        let end = self.path.rfind('.').unwrap_or(self.path.len());

        &self.path[start..end]
    }
}
