/*
"A Data-Driven Reflectance Model",

Wojciech Matusik, Hanspeter Pfister, Matt Brand and Leonard McMillan,

ACM Transactions on Graphics 22, 3(2003), 759-769.
*/

use std::{fs::File, io::Read};

use eyre::{eyre, Result};

use crate::{
    ogl::{self, ssbo::Ssbo, texture::GlTexture},
    renderer::ibl,
};

const BRDF_SAMPLING_RES_THETA_H: i32 = 90;
const BRDF_SAMPLING_RES_THETA_D: i32 = 90;
const BRDF_SAMPLING_RES_PHI_D: i32 = 360;

/// Raw BRDF measurements data
pub struct BrdfRaw {
    pub ssbo: Ssbo<{ ogl::BRDF_DATA_BINDING }>,
    pub ibl_texture: GlTexture,
}

impl BrdfRaw {
    pub fn from_path(path: &str) -> Result<Self> {
        let mut file = File::open(path)?;

        let mut dims = [0i32; 3];
        file.read_exact(&mut bytemuck::cast_slice_mut(&mut dims))?;

        let n = dims[0] * dims[1] * dims[2];
        if n != (BRDF_SAMPLING_RES_THETA_H * BRDF_SAMPLING_RES_THETA_D * BRDF_SAMPLING_RES_PHI_D
            / 2)
        {
            return Err(eyre!("Dimensions don't match"));
        }

        let mut raw = vec![0f64; 3 * n as usize];
        file.read_exact(&mut bytemuck::cast_slice_mut(&mut raw))?;

        let ssbo = Ssbo::new(&raw);
        let ibl_texture = ibl::Ibl::compute_ibl_raw_brdf(&ssbo)?;

        Ok(Self { ssbo, ibl_texture })
    }
}
