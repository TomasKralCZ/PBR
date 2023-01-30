use std::{
    ffi::{c_void, CStr},
    ptr,
    time::Duration,
};

/// Abstraction for ordinary buffers
pub mod gl_buffer;
/// Abstraction for working with OpenGL Shaders.
pub mod shader;
/// Abstraction for shader storage buffer objects (SSBOs)
pub mod ssbo;
/// Abstraction for textures
pub mod texture;
/// Abstraction for working with OpenGL Uniform Buffers.
pub mod uniform_buffer;
/// Abstraction for working with VAOs
pub mod vao;

// Indices of the vertex attributes
pub const POSITION_INDEX: u32 = 0;
pub const NORMALS_INDEX: u32 = 1;
pub const TEXCOORDS_INDEX: u32 = 2;
pub const TANGENT_INDEX: u32 = 3;

// Texture binding ports (units)
pub const ALBEDO_PORT: u32 = 0;
pub const MR_PORT: u32 = 1;
pub const NORMAL_PORT: u32 = 2;
pub const OCCLUSION_PORT: u32 = 3;
pub const EMISSIVE_PORT: u32 = 4;

pub const CLEARCOAT_INTENSITY_PORT: u32 = 5;
pub const CLEARCOAT_ROUGHNESS_PORT: u32 = 6;
pub const CLEARCOAT_NORMAL_PORT: u32 = 7;

pub const IRRADIANCE_PORT: u32 = 8;
pub const PREFILTER_PORT: u32 = 9;
pub const BRDF_PORT: u32 = 10;
pub const RAW_BRDF_PORT: u32 = 11;

// Buffer bindings
pub const TRANSFORM_BINDING: u32 = 0;
pub const PBR_MATERIAL_BINDING: u32 = 1;
pub const LIGHTNING_BINDING: u32 = 2;
pub const SETTINGS_BINDING: u32 = 3;
pub const BRDF_MERL_BINDING: u32 = 10;
pub const BRDF_MIT_BINDING: u32 = 11;
pub const BRDF_UTIA_BINDING: u32 = 12;

pub type TextureId = u32;
pub type ProgramId = u32;
pub type ShaderId = u32;
pub type BufferId = u32;
pub type VaoId = u32;

pub fn gl_time_query<R, F: FnOnce() -> R>(label: &str, fun: F) -> R {
    let mut query_id = 0;
    unsafe {
        gl::CreateQueries(gl::TIME_ELAPSED, 1, &mut query_id);
        gl::BeginQuery(gl::TIME_ELAPSED, query_id);
    }

    let res = fun();

    let mut time = 0;
    unsafe {
        gl::EndQuery(gl::TIME_ELAPSED);

        let mut done = 0;
        while done == 0 {
            gl::GetQueryObjectiv(query_id, gl::QUERY_RESULT_AVAILABLE, &mut done);
        }

        gl::GetQueryObjectui64v(query_id, gl::QUERY_RESULT, &mut time);
    }

    println!("'{}' took: {:?}", label, Duration::from_nanos(time));

    res
}

pub fn init_debug() {
    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(gl_debug_callback), ptr::null());
        gl::DebugMessageControl(
            gl::DONT_CARE,
            gl::DONT_CARE,
            gl::DONT_CARE,
            0,
            ptr::null(),
            gl::TRUE,
        );
    };
}

/// The OpenGL debug callback.
///
/// 'extern "system"' specifies the correct ABI for all platforms
extern "system" fn gl_debug_callback(
    _src: u32,
    _typ: u32,
    id: u32,
    severity: u32,
    _len: i32,
    msg: *const i8,
    _user_param: *mut c_void,
) {
    // Buffer creation on NVidia cards
    if id == 131185 {
        return;
    }

    // Shader recompilation
    if id == 131218 {
        return;
    }

    match severity {
        gl::DEBUG_SEVERITY_NOTIFICATION => print!("OpenGL - notification: "),
        gl::DEBUG_SEVERITY_LOW => print!("OpenGL - low: "),
        gl::DEBUG_SEVERITY_MEDIUM => print!("OpenGL - medium: "),
        gl::DEBUG_SEVERITY_HIGH => print!("OpenGL - high: "),
        _ => unreachable!("Unknown severity in glDebugCallback: '{}'", severity),
    }

    // TODO: check if the message is guaranteed to be ASCII
    let msg = unsafe { CStr::from_ptr(msg) };
    println!("OpenGL debug message: '{}'", msg.to_string_lossy())
}
