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
    local_size_xy: 8,
    local_size_z: 1,
};
