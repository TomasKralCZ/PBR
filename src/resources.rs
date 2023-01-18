use eyre::{eyre, Result};

use crate::{brdf_raw::BrdfRaw, ogl, scene::Scene, util::timed_scope};

pub struct Resources {
    pub scenes: Vec<LazyResource<Scene>>,
    pub merl_brdfs: Vec<LazyResource<BrdfRaw<{ ogl::BRDF_MERL_BINDING }>>>,
    pub mit_brdfs: Vec<LazyResource<BrdfRaw<{ ogl::BRDF_MIT_BINDING }>>>,
    pub utia_brdfs: Vec<LazyResource<BrdfRaw<{ ogl::BRDF_UTIA_BINDING }>>>,
}

impl Resources {
    /// Adds models to the scene
    pub fn init() -> Result<Self> {
        let mut scenes = Vec::new();

        let mut add_scene = |path: &'static str| {
            let lazy_model = LazyResource::new(path.to_string());
            scenes.push(lazy_model);
        };

        add_scene("resources/gltf/helmet/DamagedHelmet.gltf");
        add_scene("resources/gltf/MetalRoughSpheres/glTF-Binary/MetalRoughSpheres.glb");
        add_scene("resources/gltf/NewSky_PolarFacilityMap.glb");

        add_scene("resources/gltf/shoe_with_clearcoat/shoe.gltf");
        add_scene("resources/gltf/ClearCoatTest.glb");

        add_scene("resources/gltf/Sphere.glb");
        add_scene("resources/gltf/Cylinder.gltf");

        let mut merl_brdfs = Vec::new();
        for brdf in globwalk::glob("resources/BRDFDatabase/brdfs/*.binary").unwrap() {
            if let Ok(brdf) = brdf {
                if let Some(p) = brdf.path().to_str().map(|s| s.to_string()) {
                    let brdf = LazyResource::new(p);
                    merl_brdfs.push(brdf);
                }
            }
        }

        let mut mit_brdfs = Vec::new();

        let mut add_mit_brdf = |path: &'static str| {
            let brdf = LazyResource::new(path.to_string());
            mit_brdfs.push(brdf);
        };

        add_mit_brdf("resources/MITBRDFs/brushed_alum.dat");
        add_mit_brdf("resources/MITBRDFs/purple_satin.dat");
        add_mit_brdf("resources/MITBRDFs/red_velvet.dat");
        add_mit_brdf("resources/MITBRDFs/yellow_satin.dat");

        let mut utia_brdfs = Vec::new();
        for brdf in globwalk::glob("resources/UTIA/data/*.bin").unwrap() {
            if let Ok(brdf) = brdf {
                if let Some(p) = brdf.path().to_str().map(|s| s.to_string()) {
                    let brdf = LazyResource::new(p);
                    utia_brdfs.push(brdf);
                }
            }
        }

        Ok(Self {
            scenes,
            merl_brdfs,
            mit_brdfs,
            utia_brdfs,
        })
    }

    pub fn get_scene(&mut self, selected_scene: usize) -> Result<&mut Scene> {
        self.scenes[selected_scene].get()
    }

    pub fn get_merl_brdf(
        &mut self,
        selected_brdf: usize,
    ) -> Result<&mut BrdfRaw<{ ogl::BRDF_MERL_BINDING }>> {
        self.merl_brdfs[selected_brdf].get()
    }

    pub fn get_mit_brdf(
        &mut self,
        selected_brdf: usize,
    ) -> Result<&mut BrdfRaw<{ ogl::BRDF_MIT_BINDING }>> {
        self.mit_brdfs[selected_brdf].get()
    }

    pub fn get_utia_brdf(
        &mut self,
        selected_brdf: usize,
    ) -> Result<&mut BrdfRaw<{ ogl::BRDF_UTIA_BINDING }>> {
        self.utia_brdfs[selected_brdf].get()
    }

    pub fn unload(&mut self) {
        for scene in &mut self.scenes {
            scene.unload();
        }

        for scene in &mut self.merl_brdfs {
            scene.unload();
        }
    }
}

/// Loading models takes a long time, load them lazily
pub struct LazyResource<T: LoadResource> {
    path: String,
    resource: Option<T>,
}

impl<T: LoadResource> LazyResource<T> {
    fn new(path: String) -> Self {
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
            let path = &self.path;
            let resource = timed_scope(&format!("Loading '{path}'"), || T::load(&path))?;

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

impl<const BINDING: u32> LoadResource for BrdfRaw<BINDING> {
    fn load(path: &str) -> Result<Self> {
        let ext = path
            .rsplit_once(".")
            .ok_or(eyre!("BRDF file has no extension name, cannot infer type"))?
            .1;

        match ext {
            "bin" => Self::utia_from_path(path),
            "binary" => Self::merl_from_path(path),
            "dat" => Self::mit_from_path(path),
            _ => Err(eyre!("BRDF file has no extension name, cannot infer type")),
        }
    }
}
