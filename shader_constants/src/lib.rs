use serde::Serialize;

#[derive(Serialize)]
pub struct Consts {
    pub vertex_attrib_indices: VertexAttribIndices,
    pub ibl: IblConsts,
    pub texture_ports: TexturePorts,
    pub buffer_bindings: BufferBindings,
}

#[derive(Serialize)]
pub struct VertexAttribIndices {
    pub position: u32,
    pub normals: u32,
    pub texcoords: u32,
    pub tangent: u32,
}

#[derive(Serialize)]
pub struct IblConsts {
    pub cubemap_size: i32,
    pub cubemap_roughnes_levels: i32,
    pub local_size_xy: u32,
    pub local_size_z: u32,
}

#[derive(Serialize)]
pub struct TexturePorts {
    pub albedo: u32,
    pub mr: u32,
    pub normal: u32,
    pub occlusion: u32,
    pub emissive: u32,

    pub clearcoat_intensity: u32,
    pub clearcoat_roughness: u32,
    pub clearcoat_normal: u32,

    pub irradiance: u32,
    pub prefilter: u32,
    pub brdf: u32,
    pub raw_brdf: u32,
}

#[derive(Serialize)]
pub struct BufferBindings {
    pub transforms: u32,
    pub pbr_material: u32,
    pub lighting: u32,
    pub settings: u32,
    pub brdf_merl: u32,
    pub brdf_utia: u32,
}

pub const CONSTS: Consts = Consts {
    vertex_attrib_indices: VertexAttribIndices {
        position: 0,
        normals: 1,
        texcoords: 2,
        tangent: 3,
    },
    ibl: IblConsts {
        cubemap_size: 1024,
        cubemap_roughnes_levels: 7,
        /// Be careful when setting local size, is has to be smaller
        /// or equal to te smallest MIP level of the prefilter map.
        /// 4 performs much better for the prefilter shader on my GPU.
        local_size_xy: 4,
        local_size_z: 1,
    },
    texture_ports: TexturePorts {
        albedo: 0,
        mr: 1,
        normal: 2,
        occlusion: 3,
        emissive: 4,
        clearcoat_intensity: 5,
        clearcoat_roughness: 6,
        clearcoat_normal: 7,
        irradiance: 8,
        prefilter: 9,
        brdf: 10,
        raw_brdf: 11,
    },
    buffer_bindings: BufferBindings {
        transforms: 0,
        pbr_material: 1,
        lighting: 2,
        settings: 3,
        brdf_merl: 10,
        brdf_utia: 11,
    },
};
