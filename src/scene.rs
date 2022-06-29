use glam::Vec3;

use eyre::Result;

use crate::model::Model;

pub struct Scene {
    pub models: Vec<Model>,
}

impl Scene {
    /// Adds models to the scene
    pub fn init() -> Result<Self> {
        let mut models = Vec::new();

        let mut add = |path: &str| -> Result<()> {
            let start = std::time::Instant::now();

            let model = Model::from_gltf(path)?;

            let time = std::time::Instant::now().duration_since(start);
            println!("Loading '{path}' took '{time:?}'");

            models.push(model);
            Ok(())
        };

        add("resources/helmet/DamagedHelmet.gltf")?;

        add("resources/MetalRoughSpheres/glTF-Binary/MetalRoughSpheres.glb")?;

        add("resources/microphone_gxl_066_bafhcteks/microphone.gltf")?;

        add("resources/chemical_tank_-_low_poly/tank.gltf")?;

        add("resources/Sphere.glb")?;

        add("resources/bottle/WaterBottle.gltf")?;
        models.last_mut().unwrap().transform = glam::Mat4::from_scale(Vec3::splat(20.0));


        Ok(Self { models })
    }
}
