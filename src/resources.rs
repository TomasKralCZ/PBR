use eyre::Result;

use crate::{brdf_raw::BrdfRaw, scene::Scene, util::timed_scope};

pub struct Resources {
    pub scenes: Vec<LazyResource<Scene>>,
    pub brdfs: Vec<LazyResource<BrdfRaw>>,
}

impl Resources {
    /// Adds models to the scene
    pub fn init() -> Result<Self> {
        let mut scenes = Vec::new();

        let mut add_scene = |path: &'static str| {
            let lazy_model = LazyResource::new(path);
            scenes.push(lazy_model);
        };

        add_scene("resources/gltf/helmet/DamagedHelmet.gltf");
        add_scene("resources/gltf/MetalRoughSpheres/glTF-Binary/MetalRoughSpheres.glb");
        add_scene("resources/gltf/NewSky_PolarFacilityMap.glb");

        add_scene("resources/gltf/shoe_with_clearcoat/shoe.gltf");
        add_scene("resources/gltf/ClearCoatTest.glb");

        add_scene("resources/gltf/Sphere.glb");
        add_scene("resources/gltf/Cylinder.gltf");

        let mut brdfs = Vec::new();

        let mut add_brdf = |path: &'static str| {
            let brdf = LazyResource::new(path);
            brdfs.push(brdf);
        };

        add_brdf("resources/BRDFDatabase/brdfs/black-fabric.binary");
        add_brdf("resources/BRDFDatabase/brdfs/blue-acrylic.binary");
        add_brdf("resources/BRDFDatabase/brdfs/brass.binary");
        add_brdf("resources/BRDFDatabase/brdfs/fruitwood-241.binary");
        add_brdf("resources/BRDFDatabase/brdfs/gold-metallic-paint.binary");
        add_brdf("resources/BRDFDatabase/brdfs/green-latex.binary");
        add_brdf("resources/BRDFDatabase/brdfs/nylon.binary");
        add_brdf("resources/BRDFDatabase/brdfs/red-plastic.binary");
        add_brdf("resources/BRDFDatabase/brdfs/chrome.binary");

        Ok(Self { scenes, brdfs })
    }

    pub fn get_selected_scene(&mut self, selected_scene: usize) -> Result<&mut Scene> {
        self.scenes[selected_scene].get()
    }

    pub fn get_selected_brdf(&mut self, selected_brdf: usize) -> Result<&mut BrdfRaw> {
        self.brdfs[selected_brdf].get()
    }

    pub fn unload(&mut self) {
        for scene in &mut self.scenes {
            scene.unload();
        }

        for scene in &mut self.brdfs {
            scene.unload();
        }
    }
}

/// Loading models takes a long time, load them lazily
pub struct LazyResource<T: LoadResource> {
    path: &'static str,
    resource: Option<T>,
}

impl<T: LoadResource> LazyResource<T> {
    fn new(path: &'static str) -> Self {
        Self {
            path,
            resource: None,
        }
    }

    fn get(&mut self) -> Result<&mut T> {
        // Can't use if let Some(...) because the borrow checker is angry
        // Can't use get_or_insert_with(...) because I need error handling
        if self.resource.is_some() {
            Ok(self.resource.as_mut().unwrap())
        } else {
            let path = self.path;

            let resource = timed_scope(&format!("Loading '{path}'"), || T::load(path))?;

            self.resource = Some(resource);

            Ok(self.resource.as_mut().unwrap())
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
        self.resource = None;
    }
}

pub trait LoadResource: Sized {
    fn load(path: &str) -> Result<Self>;
}

impl LoadResource for Scene {
    fn load(path: &str) -> Result<Self> {
        Self::from_gltf(path)
    }
}

impl LoadResource for BrdfRaw {
    fn load(path: &str) -> Result<Self> {
        Self::from_path(path)
    }
}
