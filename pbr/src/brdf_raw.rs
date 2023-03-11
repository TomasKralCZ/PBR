use std::{fs::File, io::Read};

use eyre::{eyre, Result};

use crate::ogl::{shader::shader_permutations::ShaderDefines, ssbo::Ssbo};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum BrdfType {
    Merl,
    Utia,
}

impl BrdfType {
    pub fn to_str(self) -> &'static str {
        match self {
            BrdfType::Merl => "MERL_BRDF",
            BrdfType::Utia => "UTIA_BRDF",
        }
    }
}

impl ShaderDefines for BrdfType {
    fn defines(&self) -> Vec<&str> {
        vec![self.to_str()]
    }
}

pub struct BrdfRaw<const BINDING: u32> {
    pub typ: BrdfType,
    pub ssbo: Ssbo<BINDING>,
}

impl<const BINDING: u32> BrdfRaw<BINDING> {
    /// A Data-Driven Reflectance Model
    /// Wojciech Matusik, Hanspeter Pfister, Matt Brand and Leonard McMillan
    /// ACM Transactions on Graphics 22, 3(2003), 759-769
    pub fn merl_from_path(path: &str) -> Result<Self> {
        const RES_THETA_H: i32 = 90;
        const RES_THETA_D: i32 = 90;
        const RES_PHI_D: i32 = 360;

        let mut file = File::open(path)?;

        let mut dims = [0i32; 3];
        file.read_exact(bytemuck::cast_slice_mut(&mut dims))?;

        let samples = dims[0] * dims[1] * dims[2];
        if samples != (RES_THETA_H * RES_THETA_D * RES_PHI_D / 2) {
            return Err(eyre!("Dimensions don't match"));
        }

        let mut raw = vec![0f64; 3 * samples as usize];
        file.read_exact(bytemuck::cast_slice_mut(&mut raw))?;

        let ssbo = Ssbo::new(&raw);

        Ok(Self {
            typ: BrdfType::Merl,
            ssbo,
        })
    }

    /// Template-Based Sampling of Anisotropic BRDFs
    /// Filip J., Vavra R.
    /// Computer Graphics Forum (Proceedings of Pacific Graphics 2014, Seoul, Korea), Eurographics 2014
    pub fn utia_from_path(path: &str) -> Result<Self> {
        const STEP_P: f32 = 7.5;
        const NTI: i32 = 6;
        const NTV: i32 = 6;
        const NPI: i32 = (360. / STEP_P) as i32;
        const NPV: i32 = (360. / STEP_P) as i32;
        const PLANES: i32 = 3;

        let mut file = File::open(path)?;

        let dim = PLANES * NTI * NPI * NTV * NPV;
        let mut raw = vec![0f64; dim as usize];
        file.read_exact(bytemuck::cast_slice_mut(&mut raw))?;

        let ssbo = Ssbo::new(&raw);

        Ok(Self {
            typ: BrdfType::Utia,
            ssbo,
        })
    }
}
