use std::ffi::{c_void, CStr};
use std::ptr::{copy_nonoverlapping, with_exposed_provenance, with_exposed_provenance_mut};
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
use web_sys::*;

// -------------------- Primitive Types --------------------

type GLboolean = u8;
type GLint = i32;
type GLuint = u32;
type GLfloat = f32;
type GLsizei = i32;
type GLenum = u32;
type GLbitfield = u32;
type GLsizeiptr = i32;
type GLintptr = i32;

// -------------------- WebGL Constants --------------------

const GL_MAJOR_VERSION: GLenum = 0x821B;
const GL_MINOR_VERSION: GLenum = 0x821C;
const GL_INFO_LOG_LENGTH: GLenum = 0x8B84;
const GL_INVALID_INDEX: GLuint = 0xFFFFFFFF;
const GL_ARRAY_BUFFER: GLenum = 0x8892;
const GL_ARRAY_BUFFER_BINDING: GLenum = 0x8894;

// -------------------- WebGL State --------------------

// Maintain a single static pointer to context
static CONTEXT_PTR: AtomicPtr<WebGl2RenderingContext> = AtomicPtr::new(std::ptr::null_mut());

// Shadow state for GL bindings that WebGL returns as JS objects (not integers)
static ARRAY_BUFFER_BINDING: AtomicU32 = AtomicU32::new(0);

/// Returns the stored WebGL2 context pointer as a raw `*mut c_void`.
///
/// ThorVG's GL renderer compares the value stored via `set_gl_target` against
/// the value returned by `emscripten_webgl_get_current_context()` — they must
/// be the same pointer for ThorVG to mark the context as current.
pub fn context_ptr() -> *mut std::ffi::c_void {
    CONTEXT_PTR.load(Ordering::Relaxed) as *mut std::ffi::c_void
}

/// Sets the WebGL2 rendering context for the GL stubs (single-context legacy API).
pub fn set_webgl_context(context: WebGl2RenderingContext) {
    let ptr = CONTEXT_PTR.load(Ordering::Relaxed);

    if !ptr.is_null() {
        unsafe {
            if *ptr != context {
                let old_context = Box::from_raw(ptr);
                drop(old_context);
            } else {
                return;
            }
        }
    }

    // Store the context in a Box to prevent moves
    let context_box = Box::new(context);
    // Convert Box to raw pointer and store expose_provenanceess in atomic
    let ptr = Box::into_raw(context_box);
    CONTEXT_PTR.store(ptr, Ordering::Relaxed);
}

/// Store a WebGL context in a heap-allocated Box, returning the raw pointer.
/// Does NOT set it as the current context — call `make_current` for that.
/// The caller is responsible for calling `drop_stored_context` when done.
pub fn store_context(context: WebGl2RenderingContext) -> *mut WebGl2RenderingContext {
    Box::into_raw(Box::new(context))
}

/// Set the given pointer as the active WebGL context.
/// All subsequent GL stub calls will use this context.
pub fn make_current(ptr: *mut WebGl2RenderingContext) {
    CONTEXT_PTR.store(ptr, Ordering::Relaxed);
}

/// Drop a stored WebGL context. The pointer must have been returned by `store_context`.
/// If this context is the current active one, the global is cleared.
pub unsafe fn drop_stored_context(ptr: *mut WebGl2RenderingContext) {
    if !ptr.is_null() {
        // Clear the global if it points to the context being dropped
        let _ = CONTEXT_PTR.compare_exchange(
            ptr,
            std::ptr::null_mut(),
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
        drop(Box::from_raw(ptr));
    }
}

// Fast context access
fn get_context() -> &'static WebGl2RenderingContext {
    // Get pointer from atomic - this is very fast
    let ptr = CONTEXT_PTR.load(Ordering::Relaxed);
    if ptr.is_null() {
        panic!("WebGL context not initialized");
    }
    unsafe { &*ptr }
}

// -------------------- WebGL Stubs --------------------

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDeleteTextures(n: GLsizei, textures: *const GLuint) {
    let state = get_context();
    let textures_slice = std::slice::from_raw_parts(textures, n as usize);

    for &texture in textures_slice {
        if texture != 0 {
            // Using exposed provenance: cast integer back to pointer
            let texture_obj = Box::from_raw(with_exposed_provenance_mut::<WebGlTexture>(
                texture as usize,
            ));
            state.delete_texture(Some(&texture_obj));
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glBindFramebuffer(target: GLenum, framebuffer: GLuint) {
    let state = get_context();

    if framebuffer == 0 {
        state.bind_framebuffer(target, None);
    } else {
        let framebuffer_ref = &*(with_exposed_provenance::<WebGlFramebuffer>(framebuffer as usize));
        state.bind_framebuffer(target, Some(framebuffer_ref));
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDeleteFramebuffers(n: GLsizei, framebuffers: *const GLuint) {
    let state = get_context();
    let framebuffers_slice = std::slice::from_raw_parts(framebuffers, n as usize);

    for &framebuffer in framebuffers_slice {
        if framebuffer != 0 && (framebuffer as usize) % 0x4 == 0 {
            let fb_obj = Box::from(with_exposed_provenance::<WebGlFramebuffer>(
                framebuffer as usize,
            ));
            state.delete_framebuffer(Some(&**fb_obj));
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDeleteRenderbuffers(n: GLsizei, renderbuffers: *const GLuint) {
    let state = get_context();
    let renderbuffers_slice = std::slice::from_raw_parts(renderbuffers, n as usize);

    for &renderbuffer in renderbuffers_slice {
        if renderbuffer != 0 {
            let rb_obj = Box::from(with_exposed_provenance::<WebGlRenderbuffer>(
                renderbuffer as usize,
            ));
            state.delete_renderbuffer(Some(&**rb_obj));
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGenFramebuffers(n: GLsizei, framebuffers: *mut GLuint) {
    let state = get_context();
    let framebuffers_slice = std::slice::from_raw_parts_mut(framebuffers, n as usize);

    for framebuffer in framebuffers_slice {
        let fb = state.create_framebuffer().unwrap();
        let fb_ptr = Box::into_raw(Box::new(fb));
        *framebuffer = fb_ptr as *const _ as usize as GLuint;
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGenRenderbuffers(n: GLsizei, renderbuffers: *mut GLuint) {
    let state = get_context();
    let renderbuffers_slice = std::slice::from_raw_parts_mut(renderbuffers, n as usize);

    for renderbuffer in renderbuffers_slice {
        let rb = state.create_renderbuffer().unwrap();
        let rb_ptr = Box::into_raw(Box::new(rb));
        *renderbuffer = rb_ptr.expose_provenance() as GLuint;
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glBindRenderbuffer(target: GLenum, renderbuffer: GLuint) {
    let state = get_context();

    if renderbuffer == 0 {
        state.bind_renderbuffer(target, None);
    } else {
        let rb_ref = &*(with_exposed_provenance::<WebGlRenderbuffer>(renderbuffer as usize));
        state.bind_renderbuffer(target, Some(rb_ref));
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glRenderbufferStorageMultisample(
    target: GLenum,
    samples: GLsizei,
    internalformat: GLenum,
    width: GLsizei,
    height: GLsizei,
) {
    get_context().renderbuffer_storage_multisample(target, samples, internalformat, width, height);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glFramebufferRenderbuffer(
    target: GLenum,
    attachment: GLenum,
    renderbuffertarget: GLenum,
    renderbuffer: GLuint,
) {
    let state = get_context();

    if renderbuffer == 0 {
        state.framebuffer_renderbuffer(target, attachment, renderbuffertarget, None);
    } else {
        let rb_ref = &*(with_exposed_provenance::<WebGlRenderbuffer>(renderbuffer as usize));
        state.framebuffer_renderbuffer(target, attachment, renderbuffertarget, Some(rb_ref));
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGenTextures(n: GLsizei, textures: *mut GLuint) {
    let state = get_context();
    let textures_slice = std::slice::from_raw_parts_mut(textures, n as usize);

    for texture in textures_slice {
        // Still need to use Box for correct WebGL object management
        // But we'll batch creation when possible in real-world usage
        let tex = state.create_texture().unwrap();
        let tex_ptr = Box::into_raw(Box::new(tex));
        *texture = tex_ptr.expose_provenance() as GLuint;
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glBindTexture(target: GLenum, texture: GLuint) {
    // Fast path - most common case
    if texture == 0 {
        get_context().bind_texture(target, None);
        return;
    }

    // Only dereference if we have a non-zero texture
    let tex_ref = &*(with_exposed_provenance::<WebGlTexture>(texture as usize));
    get_context().bind_texture(target, Some(tex_ref));
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glTexImage2D(
    target: GLenum,
    level: GLint,
    internalformat: GLint,
    width: GLsizei,
    height: GLsizei,
    border: GLint,
    format: GLenum,
    type_: GLenum,
    pixels: *const c_void,
) {
    let state = get_context();
    let pixels_slice = (!pixels.is_null())
        .then(|| std::slice::from_raw_parts(pixels as *const u8, (width * height * 4) as usize));
    state
        .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            target,
            level,
            internalformat,
            width,
            height,
            border,
            format,
            type_,
            pixels_slice,
        )
        .unwrap();
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint) {
    get_context().tex_parameteri(target, pname, param);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glFramebufferTexture2D(
    target: GLenum,
    attachment: GLenum,
    textarget: GLenum,
    texture: GLuint,
    level: GLint,
) {
    let state = get_context();

    if texture == 0 {
        state.framebuffer_texture_2d(target, attachment, textarget, None, level);
    } else {
        let tex_ref = &*(with_exposed_provenance::<WebGlTexture>(texture as usize));
        state.framebuffer_texture_2d(target, attachment, textarget, Some(tex_ref), level);
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glBufferData(
    target: GLenum,
    size: GLsizeiptr,
    data: *const c_void,
    usage: GLenum,
) {
    let state = get_context();

    if !data.is_null() {
        let data_slice = std::slice::from_raw_parts(data as *const u8, size as usize);
        state.buffer_data_with_u8_array(target, data_slice, usage);
    } else {
        state.buffer_data_with_i32(target, size, usage);
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGenBuffers(n: GLsizei, buffers: *mut GLuint) {
    let state = get_context();
    let buffers_slice = std::slice::from_raw_parts_mut(buffers, n as usize);

    for buffer in buffers_slice {
        let buf = state.create_buffer().unwrap();
        let buf_ptr = Box::into_raw(Box::new(buf));
        *buffer = buf_ptr.expose_provenance() as GLuint;
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glBindBuffer(target: GLenum, buffer: GLuint) {
    let state = get_context();

    if target == GL_ARRAY_BUFFER {
        ARRAY_BUFFER_BINDING.store(buffer, Ordering::Relaxed);
    }

    if buffer == 0 {
        state.bind_buffer(target, None);
    } else {
        let buf_ref = &*(with_exposed_provenance::<WebGlBuffer>(buffer as usize));
        state.bind_buffer(target, Some(buf_ref));
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDeleteBuffers(n: GLsizei, buffers: *const GLuint) {
    let state = get_context();
    let buffers_slice = std::slice::from_raw_parts(buffers, n as usize);

    for &buffer in buffers_slice {
        if buffer != 0 {
            let buf_obj =
                Box::from_raw(with_exposed_provenance_mut::<WebGlBuffer>(buffer as usize));
            state.delete_buffer(Some(&buf_obj));
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGenVertexArrays(n: GLsizei, arrays: *mut GLuint) {
    let state = get_context();
    let arrays_slice = std::slice::from_raw_parts_mut(arrays, n as usize);

    for array in arrays_slice {
        let vao = state.create_vertex_array().unwrap();
        let vao_ptr = Box::into_raw(Box::new(vao));
        *array = vao_ptr.expose_provenance() as GLuint;
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glBindVertexArray(array: GLuint) {
    let state = get_context();

    if array == 0 {
        state.bind_vertex_array(None);
    } else {
        let vao_ref = &*(with_exposed_provenance::<WebGlVertexArrayObject>(array as usize));
        state.bind_vertex_array(Some(vao_ref));
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDeleteVertexArrays(n: GLsizei, arrays: *const GLuint) {
    let state = get_context();
    let arrays_slice = std::slice::from_raw_parts(arrays, n as usize);

    for &array in arrays_slice {
        if array != 0 {
            let vao_obj = Box::from_raw(with_exposed_provenance_mut::<WebGlVertexArrayObject>(
                array as usize,
            ));
            state.delete_vertex_array(Some(&vao_obj));
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGetIntegerv(pname: GLenum, data: *mut GLint) {
    let state = get_context();
    match pname {
        GL_MAJOR_VERSION => {
            *data = 3;
        }
        GL_MINOR_VERSION => {
            *data = 0;
        }
        GL_ARRAY_BUFFER_BINDING => {
            *data = ARRAY_BUFFER_BINDING.load(Ordering::Relaxed) as GLint;
        }
        _ => {
            // WebGL binding queries return JS objects, not numbers.
            // Fall back to 0 for unhandled ones.
            *data = state
                .get_parameter(pname)
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as GLint;
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glCreateShader(type_: GLenum) -> GLuint {
    let state = get_context();
    let shader = state.create_shader(type_).unwrap();
    let shader_ptr = Box::into_raw(Box::new(shader));
    shader_ptr.expose_provenance() as GLuint
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glShaderSource(
    shader: GLuint,
    count: GLsizei,
    string: *const *const i8,
    length: *const GLint,
) {
    if shader == 0 {
        return;
    }

    let state = get_context();
    let shader_ref = &*(with_exposed_provenance::<WebGlShader>(shader as usize));

    let strings = std::slice::from_raw_parts(string, count as usize);
    let lengths = if length.is_null() {
        None
    } else {
        Some(std::slice::from_raw_parts(length, count as usize))
    };
    let sources = strings
        .iter()
        .enumerate()
        .map(|(i, &ptr)| {
            let len = lengths.map(|lens| lens[i]).unwrap_or(0);
            if len > 0 {
                let bytes = std::slice::from_raw_parts(ptr as *const u8, len as usize);
                std::str::from_utf8_unchecked(bytes)
            } else {
                CStr::from_ptr(ptr).to_str().unwrap_or_default()
            }
        })
        .collect::<Vec<_>>();
    let source = sources.join("").replace("\\\n", "\n");
    state.shader_source(shader_ref, &source);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glCompileShader(shader: GLuint) {
    let state = get_context();

    if shader != 0 {
        let shader_ref = &*(with_exposed_provenance::<WebGlShader>(shader as usize));
        state.compile_shader(shader_ref);
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGetShaderiv(shader: GLuint, pname: GLenum, params: *mut GLint) {
    let state = get_context();

    if shader != 0 {
        let shader_ref = &*(with_exposed_provenance::<WebGlShader>(shader as usize));
        match pname {
            GL_INFO_LOG_LENGTH => {
                *params = state
                    .get_shader_info_log(shader_ref)
                    .map(|log| log.into_bytes().len() as GLint)
                    .unwrap_or_default();
            }
            WebGl2RenderingContext::COMPILE_STATUS => {
                *params = state
                    .get_shader_parameter(shader_ref, pname)
                    .as_bool()
                    .unwrap() as GLint;
            }
            _ => {
                *params = state
                    .get_shader_parameter(shader_ref, pname)
                    .as_f64()
                    .unwrap() as GLint;
            }
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGetShaderInfoLog(
    shader: GLuint,
    buf_size: GLsizei,
    length: *mut GLint,
    info_log: *mut i8,
) {
    let state = get_context();

    if shader != 0 {
        let shader_ref = &*(with_exposed_provenance::<WebGlShader>(shader as usize));
        if let Some(log) = state.get_shader_info_log(shader_ref) {
            let bytes = log.into_bytes();
            let len = bytes.len().min(buf_size as usize - 1);
            copy_nonoverlapping(bytes.as_ptr() as *const i8, info_log, len);
            *info_log.add(len) = 0;
            if !length.is_null() {
                *length = len as GLint;
            }
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDeleteShader(shader: GLuint) {
    let state = get_context();

    if shader != 0 {
        let shader_obj = Box::from_raw(with_exposed_provenance_mut::<WebGlShader>(shader as usize));
        state.delete_shader(Some(&shader_obj));
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glCreateProgram() -> GLuint {
    let state = get_context();
    let program = state.create_program().unwrap();
    let program_ptr = Box::into_raw(Box::new(program));
    program_ptr.expose_provenance() as GLuint
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glAttachShader(program: GLuint, shader: GLuint) {
    let state = get_context();

    if program != 0 && shader != 0 {
        let program_ref = &*(with_exposed_provenance::<WebGlProgram>(program as usize));
        let shader_ref = &*(with_exposed_provenance::<WebGlShader>(shader as usize));
        state.attach_shader(program_ref, shader_ref);
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glLinkProgram(program: GLuint) {
    let state = get_context();

    if program != 0 {
        let program_ref = &*(with_exposed_provenance::<WebGlProgram>(program as usize));
        state.link_program(program_ref);
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGetProgramiv(program: GLuint, pname: GLenum, params: *mut GLint) {
    let state = get_context();

    if program != 0 {
        let program_ref = &*(with_exposed_provenance::<WebGlProgram>(program as usize));
        match pname {
            GL_INFO_LOG_LENGTH => {
                *params = state
                    .get_program_info_log(program_ref)
                    .map(|log| log.into_bytes().len() as GLint)
                    .unwrap_or_default();
            }
            WebGl2RenderingContext::LINK_STATUS => {
                *params = state
                    .get_program_parameter(program_ref, pname)
                    .as_bool()
                    .unwrap() as GLint;
            }
            _ => {
                *params = state
                    .get_program_parameter(program_ref, pname)
                    .as_f64()
                    .unwrap() as GLint;
            }
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGetProgramInfoLog(
    program: GLuint,
    buf_size: GLsizei,
    length: *mut GLint,
    info_log: *mut i8,
) {
    let state = get_context();

    if program != 0 {
        let program_ref = &*(with_exposed_provenance::<WebGlProgram>(program as usize));
        if let Some(log) = state.get_program_info_log(program_ref) {
            let bytes = log.into_bytes();
            let len = bytes.len().min(buf_size as usize - 1);
            copy_nonoverlapping(bytes.as_ptr() as *const i8, info_log, len);
            *info_log.add(len) = 0;
            if !length.is_null() {
                *length = len as GLint;
            }
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDeleteProgram(program: GLuint) {
    let state = get_context();

    if program != 0 {
        let program_obj = Box::from_raw(with_exposed_provenance_mut::<WebGlProgram>(
            program as usize,
        ));
        state.delete_program(Some(&program_obj));
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glUseProgram(program: GLuint) {
    let state = get_context();

    if program == 0 {
        state.use_program(None);
    } else {
        let program_ref = &*(with_exposed_provenance::<WebGlProgram>(program as usize));
        state.use_program(Some(program_ref));
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGetUniformLocation(program: GLuint, name: *const i8) -> GLint {
    let state = get_context();

    if program != 0 {
        let program_ref = &*(with_exposed_provenance::<WebGlProgram>(program as usize));
        let name_str = CStr::from_ptr(name).to_str().unwrap_or_default();

        if let Some(location) = state.get_uniform_location(program_ref, name_str) {
            let location_ptr = Box::into_raw(Box::new(location));
            return location_ptr.expose_provenance() as GLint;
        }
    }

    -1 // Location not found
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glGetUniformBlockIndex(
    program: GLuint,
    uniform_block_name: *const i8,
) -> GLuint {
    let state = get_context();

    if program != 0 {
        let program_ref = &*(with_exposed_provenance::<WebGlProgram>(program as usize));
        let name = CStr::from_ptr(uniform_block_name)
            .to_str()
            .unwrap_or_default();
        return state.get_uniform_block_index(program_ref, name);
    }

    GL_INVALID_INDEX
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glUniform1iv(location: GLint, count: GLsizei, value: *const GLint) {
    let state = get_context();

    if location >= 0 && !value.is_null() && count > 0 {
        let location_ref = &*(with_exposed_provenance::<WebGlUniformLocation>(location as usize));
        let values = std::slice::from_raw_parts(value, count as usize);

        if count == 1 {
            state.uniform1i(Some(location_ref), values[0]);
        } else {
            state.uniform1iv_with_i32_array(Some(location_ref), values);
        }
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glEnableVertexAttribArray(index: GLuint) {
    get_context().enable_vertex_attrib_array(index);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDisableVertexAttribArray(index: GLuint) {
    get_context().disable_vertex_attrib_array(index);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glVertexAttrib4f(
    index: GLuint,
    x: GLfloat,
    y: GLfloat,
    z: GLfloat,
    w: GLfloat,
) {
    get_context().vertex_attrib4f(index, x, y, z, w);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glVertexAttribPointer(
    index: GLuint,
    size: GLint,
    type_: GLenum,
    normalized: GLboolean,
    stride: GLsizei,
    pointer: *const c_void,
) {
    get_context().vertex_attrib_pointer_with_i32(
        index,
        size,
        type_,
        normalized != 0,
        stride,
        pointer as i32,
    );
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glActiveTexture(texture: GLenum) {
    get_context().active_texture(texture);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glUniformBlockBinding(
    program: GLuint,
    uniform_block_index: GLuint,
    uniform_block_binding: GLuint,
) {
    let state = get_context();

    if program != 0 {
        let program_ref = &*(with_exposed_provenance::<WebGlProgram>(program as usize));
        state.uniform_block_binding(program_ref, uniform_block_index, uniform_block_binding);
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glBindBufferRange(
    target: GLenum,
    index: GLuint,
    buffer: GLuint,
    offset: GLintptr,
    size: GLsizeiptr,
) {
    let state = get_context();

    if buffer != 0 {
        let buf_ref = &*(with_exposed_provenance::<WebGlBuffer>(buffer as usize));
        state.bind_buffer_range_with_i32_and_i32(target, index, Some(buf_ref), offset, size);
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDrawElements(
    mode: GLenum,
    count: GLsizei,
    type_: GLenum,
    indices: *const c_void,
) {
    get_context().draw_elements_with_i32(mode, count, type_, indices as i32);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glEnable(cap: GLenum) {
    get_context().enable(cap);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDisable(cap: GLenum) {
    get_context().disable(cap);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    get_context().viewport(x, y, width, height);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glClearColor(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
    get_context().clear_color(red, green, blue, alpha);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glClearStencil(s: GLint) {
    get_context().clear_stencil(s);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glClearDepthf(d: GLfloat) {
    get_context().clear_depth(d);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDepthMask(flag: GLboolean) {
    get_context().depth_mask(flag != 0);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glClear(mask: GLbitfield) {
    get_context().clear(mask);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glBlitFramebuffer(
    src_x0: GLint,
    src_y0: GLint,
    src_x1: GLint,
    src_y1: GLint,
    dst_x0: GLint,
    dst_y0: GLint,
    dst_x1: GLint,
    dst_y1: GLint,
    mask: GLbitfield,
    filter: GLenum,
) {
    get_context().blit_framebuffer(
        src_x0, src_y0, src_x1, src_y1, dst_x0, dst_y0, dst_x1, dst_y1, mask, filter,
    );
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glBlendFunc(sfactor: GLenum, dfactor: GLenum) {
    get_context().blend_func(sfactor, dfactor);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glBlendEquation(mode: GLenum) {
    get_context().blend_equation(mode);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glCullFace(mode: GLenum) {
    get_context().cull_face(mode);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glFrontFace(mode: GLenum) {
    get_context().front_face(mode);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glDepthFunc(func: GLenum) {
    get_context().depth_func(func);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glInvalidateFramebuffer(
    target: GLenum,
    num_attachments: GLsizei,
    attachments: *const GLenum,
) {
    let state = get_context();
    let attachments_slice = std::slice::from_raw_parts(attachments, num_attachments as usize);
    let js_array = js_sys::Array::new();
    for &attachment in attachments_slice {
        js_array.push(&attachment.into());
    }
    state.invalidate_framebuffer(target, &js_array).unwrap();
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glUniform1f(location: GLint, v0: GLfloat) {
    if location >= 0 {
        let location_ref = &*(with_exposed_provenance::<WebGlUniformLocation>(location as usize));
        get_context().uniform1f(Some(location_ref), v0);
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glUniformMatrix3fv(
    location: GLint,
    count: GLsizei,
    _transpose: GLboolean,
    value: *const GLfloat,
) {
    if location >= 0 {
        let data = std::slice::from_raw_parts(value, (count * 9) as usize);
        let location_ref = &*(with_exposed_provenance::<WebGlUniformLocation>(location as usize));
        // WebGL always transposes column-major; ThorVG passes row-major matrices
        // so transpose=true (1) is the norm, but we forward as-is since ThorVG
        // already handles the layout.
        get_context().uniform_matrix3fv_with_f32_array(Some(location_ref), false, data);
    }
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glScissor(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    get_context().scissor(x, y, width, height);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glStencilFunc(func: GLenum, ref_: GLint, mask: GLuint) {
    get_context().stencil_func(func, ref_, mask);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glStencilOp(fail: GLenum, zfail: GLenum, zpass: GLenum) {
    get_context().stencil_op(fail, zfail, zpass);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glStencilFuncSeparate(
    face: GLenum,
    func: GLenum,
    ref_: GLint,
    mask: GLuint,
) {
    get_context().stencil_func_separate(face, func, ref_, mask);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glStencilOpSeparate(
    face: GLenum,
    sfail: GLenum,
    dpfail: GLenum,
    dppass: GLenum,
) {
    get_context().stencil_op_separate(face, sfail, dpfail, dppass);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn glColorMask(
    red: GLboolean,
    green: GLboolean,
    blue: GLboolean,
    alpha: GLboolean,
) {
    get_context().color_mask(red != 0, green != 0, blue != 0, alpha != 0);
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn emscripten_webgl_get_current_context() -> *mut WebGl2RenderingContext {
    CONTEXT_PTR.load(Ordering::Relaxed)
}

#[cfg_attr(feature = "wasm", no_mangle)]
pub unsafe extern "C" fn emscripten_webgl_make_context_current(ctx: *mut c_void) -> i32 {
    if !ctx.is_null() {
        CONTEXT_PTR.store(ctx as *mut WebGl2RenderingContext, Ordering::Relaxed);
    }
    0 // success (EMSCRIPTEN_RESULT = int, 0 means OK)
}
