use std::{
    ffi::{c_void, CStr},
    mem::size_of,
    ptr,
};

/// Abstraction for working with OpenGL Shaders.
pub mod shader;

/// Abstraction for working with OpenGL Uniform Buffers.
pub mod uniform_buffer;

// Indices of the vertex attributes
pub const POSITION_INDEX: u32 = 0;
pub const NORMALS_INDEX: u32 = 1;
pub const TEXCOORDS_INDEX: u32 = 2;

// Texture binding ports (units)
pub const ALBEDO_PORT: u32 = 0;
pub const MR_PORT: u32 = 1;
pub const NORMAL_PORT: u32 = 2;
pub const OCCLUSION_PORT: u32 = 3;
pub const EMISSIVE_PORT: u32 = 4;
pub const IRRADIANCE_PORT: u32 = 5;
pub const PREFILTER_PORT: u32 = 6;
pub const BRDF_PORT: u32 = 7;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct QuadVertex {
    pub pos: [f32; 3],
    pub texcoords: [f32; 2],
}

#[rustfmt::skip]
pub const QUAD_VERTICES: [QuadVertex; 4] = [
    QuadVertex {pos: [-1., 1., 0.],texcoords: [0., 1.] },
    QuadVertex {pos: [-1., -1., 0.],texcoords: [0., 0.] },
    QuadVertex {pos: [1., 1., 0.],texcoords: [1., 1.] },
    QuadVertex {pos: [1., -1., 0.],texcoords: [1., 0.] },
];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CubeVertex {
    pos: [f32; 3],
}

#[rustfmt::skip]
pub const CUBE_VERTICES: [CubeVertex; 8] = [
    CubeVertex{pos: [-1., -1., 1.]},
    CubeVertex{pos: [1., -1., 1.]},
    CubeVertex{pos: [1., 1., 1.]},
    CubeVertex{pos: [-1., 1., 1.]},
    CubeVertex{pos: [-1., -1., -1.]},
    CubeVertex{pos: [1., -1., -1.]},
    CubeVertex{pos: [1., 1., -1.]},
    CubeVertex{pos: [-1., 1., -1.]},
];

#[rustfmt::skip]
pub const CUBE_INDICES: [u8; 36] = [
    0, 2, 1,
    0, 3, 2,
    1, 6, 5,
    1, 2, 6,
    5, 7, 4,
    5, 6, 7,
    4, 3, 0,
    4, 7, 3,
    3, 7, 6,
    3, 6, 2,
    4, 0, 1,
    4, 1, 5,
];

/// Attach a float buffer to a VAO
pub fn attach_float_buf<T: bytemuck::Pod + bytemuck::Zeroable>(
    vao: u32,
    buffer: &[T],
    components: i32,
    attrib_index: u32,
    typ: u32,
) -> u32 {
    let mut buf_id: u32 = 0;

    unsafe {
        gl::CreateBuffers(1, &mut buf_id);

        let bytes = bytemuck::cast_slice::<T, u8>(buffer);

        gl::NamedBufferStorage(
            buf_id,
            bytes.len() as isize,
            bytes.as_ptr() as _,
            gl::DYNAMIC_STORAGE_BIT,
        );

        // attrib_index is implicitly the same as the vertex buffer binding !
        gl::VertexArrayVertexBuffer(vao, attrib_index, buf_id, 0, size_of::<T>() as i32);

        gl::EnableVertexArrayAttrib(vao, attrib_index);
        gl::VertexArrayAttribFormat(vao, attrib_index, components, typ, gl::FALSE, 0 as _);
        gl::VertexArrayAttribBinding(vao, attrib_index, attrib_index);
    }

    buf_id
}

/// Create an opengl buffer with floating-point content.
///
/// 'buffer' is a reference to a slice of T.
///
/// 'components', 'attrib index' and 'typ' have the same meaning as the respective
/// arguments in glVertexAttribPointer.
pub fn attach_float_buf_multiple_attribs<T: bytemuck::Pod + bytemuck::Zeroable>(
    vao: u32,
    buffer: &[T],
    componentss: &[i32],
    attrib_indexes: &[u32],
    types: &[u32],
    stride: usize,
    offsets: &[usize],
) -> u32 {
    let mut buf_id: u32 = 0;

    unsafe {
        gl::CreateBuffers(1, &mut buf_id);

        let bytes = bytemuck::cast_slice::<T, u8>(buffer);

        gl::NamedBufferStorage(
            buf_id,
            bytes.len() as isize,
            bytes.as_ptr() as _,
            gl::DYNAMIC_STORAGE_BIT,
        );

        let buf_binding = 0;

        gl::EnableVertexAttribArray(buf_binding);
        gl::VertexArrayVertexBuffer(vao, buf_binding, buf_id, 0, stride as i32);

        for i in 0..attrib_indexes.len() {
            gl::EnableVertexArrayAttrib(vao, attrib_indexes[i]);
            gl::VertexArrayAttribFormat(
                vao,
                attrib_indexes[i],
                componentss[i],
                types[i],
                gl::FALSE,
                offsets[i] as _,
            );
            gl::VertexArrayAttribBinding(vao, attrib_indexes[i], buf_binding);
        }
    }

    buf_id
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
