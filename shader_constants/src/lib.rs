use serde::Serialize;

#[derive(Serialize)]
pub struct Ibl {
    pub cubemap_size: i32,
    pub prefilter_map_roughnes_levels: i32,
    pub local_size_xy: u32,
    pub local_size_z: u32,
}

pub const IBL: Ibl = Ibl {
    cubemap_size: 1024,
    prefilter_map_roughnes_levels: 7,
    /// Be careful when setting local size, is has to be smaller
    /// or equal to te smallest MIP level of the prefilter map
    /// 4 performs much better for the prefilter shader
    local_size_xy: 4,
    local_size_z: 1,
};
