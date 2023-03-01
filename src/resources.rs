use eyre::{eyre, Result};
use shader_constants::CONSTS;

use crate::{brdf_raw::BrdfRaw, renderer::ibl::IblEnv, scene::Scene, util::timed_scope};

pub struct Resources {
    pub scenes: Vec<LazyResource<Scene>>,
    pub envmaps: Vec<LazyResource<IblEnv>>,
    pub merl_brdfs: Vec<LazyResource<BrdfRaw<{ CONSTS.buffer_bindings.brdf_merl }>>>,
    pub utia_brdfs: Vec<LazyResource<BrdfRaw<{ CONSTS.buffer_bindings.brdf_utia }>>>,
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

        add_scene("resources/gltf/shader_ball/shaderBall.glb");
        add_scene("resources/gltf/Sphere.glb");
        add_scene("resources/gltf/Cylinder.gltf");
        add_scene("resources/gltf/Cube.glb");
        add_scene("resources/gltf/RoughnessMetallicSpheres.glb");
        add_scene("resources/gltf/NormalTangentMirrorTest.glb");

        let envmaps = Self::add_glob_res("resources/IBL/**.hdr");
        let mut merl_brdfs = Self::add_glob_res("resources/BRDFDatabase/brdfs/*.binary");
        merl_brdfs.sort_by(|m1, m2| m1.name().cmp(m2.name()));

        let mut utia_brdfs = Self::add_glob_res("resources/UTIA/data/*.bin");
        utia_brdfs.sort_by(|m1, m2| m1.name().cmp(m2.name()));

        Ok(Self {
            scenes,
            envmaps,
            merl_brdfs,
            utia_brdfs,
        })
    }

    fn add_glob_res<T: LoadResource>(glob_path: &str) -> Vec<LazyResource<T>> {
        let mut resources = Vec::new();
        for res in globwalk::glob(glob_path).unwrap() {
            if let Ok(res) = res {
                if let Some(p) = res.path().to_str().map(|s| s.to_string()) {
                    let envmap = LazyResource::new(p);
                    resources.push(envmap);
                }
            }
        }

        resources
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

    pub fn load(&mut self) -> Result<&mut T> {
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

impl LoadResource for IblEnv {
    fn load(path: &str) -> Result<Self> {
        Self::from_equimap_path(path)
    }
}
