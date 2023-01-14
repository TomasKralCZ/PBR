use eyre::Result;

use crate::{scene::Scene, util::timed_scope};

pub struct Scenes {
    pub selected_scene: usize,
    pub scenes: Vec<LazyScene>,
}

impl Scenes {
    /// Adds models to the scene
    pub fn init() -> Result<Self> {
        let mut models = Vec::new();

        let mut add = |path: &'static str| {
            let lazy_model = LazyScene::new(path);
            models.push(lazy_model);
        };

        add("resources/helmet/DamagedHelmet.gltf");

        add("resources/MetalRoughSpheres/glTF-Binary/MetalRoughSpheres.glb");

        add("resources/Sphere.glb");

        add("resources/shoe_with_clearcoat/shoe.gltf");

        add("resources/ClearCoatTest.glb");

        add("resources/kasatka_71m_-_three-bolt_equipment/kasatka.gltf");

        add("resources/bottle/WaterBottle.gltf");

        add("resources/free_1975_porsche_911_930_turbo/porsche.gltf");

        add("resources/ToyCar.glb");

        add("resources/santa_conga_freebiexmass/drum.gltf");

        add("resources/Cylinder.gltf");

        add("resources/NewSky_PolarFacilityMap.glb");

        Ok(Self {
            selected_scene: 0,
            scenes: models,
        })
    }

    pub fn get_selected_scene(&mut self) -> Result<&mut Scene> {
        self.scenes[self.selected_scene].get()
    }
}

/// Loading models takes a long time, load them lazily
pub struct LazyScene {
    path: &'static str,
    scene: Option<Scene>,
}

impl LazyScene {
    fn new(path: &'static str) -> Self {
        Self { path, scene: None }
    }

    fn get(&mut self) -> Result<&mut Scene> {
        // Can't use if let Some(...) because the borrow checker is angry
        // Can't use get_or_insert_with(...) because I need error handling
        if self.scene.is_some() {
            Ok(self.scene.as_mut().unwrap())
        } else {
            let path = self.path;

            let model = timed_scope(&format!("Loading '{path}'"), || Scene::from_gltf(path))?;

            self.scene = Some(model);

            Ok(self.scene.as_mut().unwrap())
        }
    }

    pub fn name(&self) -> &str {
        // Find the index where the filename starts (if any)
        let start = self
            .path
            .rfind('/')
            .map(|i| i + 1)
            .unwrap_or_else(|| self.path.rfind('\\').map(|i| i + 1).unwrap_or(0));
        // Find the index where the file extension starts (if any)
        let end = self.path.rfind('.').unwrap_or(self.path.len());

        &self.path[start..end]
    }

    pub fn unload(&mut self) {
        self.scene = None;
    }
}
