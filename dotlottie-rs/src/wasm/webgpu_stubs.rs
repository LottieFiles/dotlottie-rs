use std::ffi::{c_char, c_void, CStr};
use std::ptr::NonNull;
use wasm_bindgen::prelude::JsValue;
use web_sys::*;

// -------------------- Primitive Types --------------------

type WGPUBool = u32;

// -------------------- Flag Types --------------------

type WGPUBufferUsage = u64;
type WGPUColorWriteMask = u64;
type WGPUShaderStage = u64;
type WGPUTextureUsage = u64;

// -------------------- Enum Types --------------------

type WGPUAddressMode = u32;
type WGPUBlendFactor = u32;
type WGPUBlendOperation = u32;
type WGPUBufferBindingType = u32;
type WGPUCompareFunction = u32;
type WGPUCullMode = u32;
type WGPUFilterMode = u32;
type WGPUFrontFace = u32;
type WGPUIndexFormat = u32;
type WGPULoadOp = u32;
type WGPUMipmapFilterMode = u32;
type WGPUOptionalBool = u32;
type WGPUPrimitiveTopology = u32;
type WGPUSamplerBindingType = u32;
type WGPUSType = u32;
type WGPUStencilOperation = u32;
type WGPUStorageTextureAccess = u32;
type WGPUStoreOp = u32;
type WGPUSurfaceGetCurrentTextureStatus = u32;
type WGPUTextureAspect = u32;
type WGPUTextureDimension = u32;
type WGPUTextureFormat = u32;
type WGPUTextureSampleType = u32;
type WGPUTextureViewDimension = u32;
type WGPUVertexFormat = u32;
type WGPUVertexStepMode = u32;

// -------------------- Enums --------------------

fn enum_wgpu_address_mode(address_mode: WGPUAddressMode) -> GpuAddressMode {
    match address_mode {
        0x00 => GpuAddressMode::__Invalid,
        0x01 => GpuAddressMode::ClampToEdge,
        0x02 => GpuAddressMode::Repeat,
        0x03 => GpuAddressMode::MirrorRepeat,
        _ => panic!("Invalid address mode: {address_mode}"),
    }
}

fn enum_wgpu_blend_factor(blend_factor: WGPUBlendFactor) -> GpuBlendFactor {
    match blend_factor {
        0x00 => GpuBlendFactor::__Invalid,
        0x01 => GpuBlendFactor::Zero,
        0x02 => GpuBlendFactor::One,
        0x03 => GpuBlendFactor::Src,
        0x04 => GpuBlendFactor::OneMinusSrc,
        0x05 => GpuBlendFactor::SrcAlpha,
        0x06 => GpuBlendFactor::OneMinusSrcAlpha,
        0x07 => GpuBlendFactor::Dst,
        0x08 => GpuBlendFactor::OneMinusDst,
        0x09 => GpuBlendFactor::DstAlpha,
        0x0A => GpuBlendFactor::OneMinusDstAlpha,
        0x0B => GpuBlendFactor::SrcAlphaSaturated,
        0x0C => GpuBlendFactor::Constant,
        0x0D => GpuBlendFactor::OneMinusConstant,
        0x0E => GpuBlendFactor::Src1,
        0x0F => GpuBlendFactor::OneMinusSrc1,
        0x10 => GpuBlendFactor::Src1Alpha,
        0x11 => GpuBlendFactor::OneMinusSrc1Alpha,
        _ => panic!("Invalid blend factor: {blend_factor}"),
    }
}

fn enum_wgpu_blend_operation(blend_operation: WGPUBlendOperation) -> GpuBlendOperation {
    match blend_operation {
        0x00 => GpuBlendOperation::__Invalid,
        0x01 => GpuBlendOperation::Add,
        0x02 => GpuBlendOperation::Subtract,
        0x03 => GpuBlendOperation::ReverseSubtract,
        0x04 => GpuBlendOperation::Min,
        0x05 => GpuBlendOperation::Max,
        _ => panic!("Invalid blend operation: {blend_operation}"),
    }
}

fn enum_wgpu_buffer_binding_type(
    buffer_binding_type: WGPUBufferBindingType,
) -> GpuBufferBindingType {
    match buffer_binding_type {
        0x00 => GpuBufferBindingType::__Invalid, // BindingNotUsed
        0x01 => GpuBufferBindingType::__Invalid, // Undefined
        0x02 => GpuBufferBindingType::Uniform,
        0x03 => GpuBufferBindingType::Storage,
        0x04 => GpuBufferBindingType::ReadOnlyStorage,
        _ => panic!("Invalid buffer binding type: {buffer_binding_type}"),
    }
}

fn enum_wgpu_compare_function(compare_function: WGPUCompareFunction) -> GpuCompareFunction {
    match compare_function {
        0x00 => GpuCompareFunction::__Invalid,
        0x01 => GpuCompareFunction::Never,
        0x02 => GpuCompareFunction::Less,
        0x03 => GpuCompareFunction::Equal,
        0x04 => GpuCompareFunction::LessEqual,
        0x05 => GpuCompareFunction::Greater,
        0x06 => GpuCompareFunction::NotEqual,
        0x07 => GpuCompareFunction::GreaterEqual,
        0x08 => GpuCompareFunction::Always,
        _ => panic!("Invalid compare function: {compare_function}"),
    }
}

fn enum_wgpu_cull_mode(cull_mode: WGPUCullMode) -> GpuCullMode {
    match cull_mode {
        0x00 => GpuCullMode::__Invalid,
        0x01 => GpuCullMode::None,
        0x02 => GpuCullMode::Front,
        0x03 => GpuCullMode::Back,
        _ => panic!("Invalid cull mode: {cull_mode}"),
    }
}

fn enum_wgpu_filter_mode(filter_mode: WGPUFilterMode) -> GpuFilterMode {
    match filter_mode {
        0x00 => GpuFilterMode::__Invalid,
        0x01 => GpuFilterMode::Nearest,
        0x02 => GpuFilterMode::Linear,
        _ => panic!("Invalid filter mode: {filter_mode}"),
    }
}

fn enum_wgpu_front_face(front_face: WGPUFrontFace) -> GpuFrontFace {
    match front_face {
        0x00 => GpuFrontFace::__Invalid,
        0x01 => GpuFrontFace::Ccw,
        0x02 => GpuFrontFace::Cw,
        _ => panic!("Invalid front face: {front_face}"),
    }
}

fn enum_wgpu_index_format(index_format: WGPUIndexFormat) -> GpuIndexFormat {
    match index_format {
        0x00 => GpuIndexFormat::__Invalid,
        0x01 => GpuIndexFormat::Uint16,
        0x02 => GpuIndexFormat::Uint32,
        _ => panic!("Invalid index format: {index_format}"),
    }
}

fn enum_wgpu_load_op(load_op: WGPULoadOp) -> GpuLoadOp {
    match load_op {
        0x00 => GpuLoadOp::__Invalid, // Undefined
        0x01 => GpuLoadOp::Load,
        0x02 => GpuLoadOp::Clear,
        _ => panic!("Invalid load op: {load_op}"),
    }
}

fn enum_wgpu_mipmap_filter_mode(mipmap_filter_mode: WGPUMipmapFilterMode) -> GpuMipmapFilterMode {
    match mipmap_filter_mode {
        0x00 => GpuMipmapFilterMode::__Invalid, // Undefined
        0x01 => GpuMipmapFilterMode::Nearest,
        0x02 => GpuMipmapFilterMode::Linear,
        _ => panic!("Invalid mipmap filter mode: {mipmap_filter_mode}"),
    }
}

fn enum_wgpu_optional_bool(optional_bool: WGPUOptionalBool) -> Option<bool> {
    match optional_bool {
        0x00 => Some(false), // False
        0x01 => Some(true),  // True
        0x02 => None,        // Undefined
        _ => panic!("Invalid optional bool: {optional_bool}"),
    }
}

fn enum_wgpu_primitive_topology(topology: WGPUPrimitiveTopology) -> GpuPrimitiveTopology {
    match topology {
        0x00 => GpuPrimitiveTopology::__Invalid, // Undefined
        0x01 => GpuPrimitiveTopology::PointList,
        0x02 => GpuPrimitiveTopology::LineList,
        0x03 => GpuPrimitiveTopology::LineStrip,
        0x04 => GpuPrimitiveTopology::TriangleList,
        0x05 => GpuPrimitiveTopology::TriangleStrip,
        _ => panic!("Invalid primitive topology: {topology}"),
    }
}

fn enum_wgpu_sampler_binding_type(
    sampler_binding_type: WGPUSamplerBindingType,
) -> GpuSamplerBindingType {
    match sampler_binding_type {
        0x00 => GpuSamplerBindingType::__Invalid, // BindingNotUsed
        0x01 => GpuSamplerBindingType::__Invalid, // Undefined
        0x02 => GpuSamplerBindingType::Filtering,
        0x03 => GpuSamplerBindingType::NonFiltering,
        0x04 => GpuSamplerBindingType::Comparison,
        _ => panic!("Invalid sampler binding type: {sampler_binding_type}"),
    }
}

fn enum_wgpu_stencil_operation(stencil_operation: WGPUStencilOperation) -> GpuStencilOperation {
    match stencil_operation {
        0x00 => GpuStencilOperation::__Invalid, // Undefined
        0x01 => GpuStencilOperation::Keep,
        0x02 => GpuStencilOperation::Zero,
        0x03 => GpuStencilOperation::Replace,
        0x04 => GpuStencilOperation::Invert,
        0x05 => GpuStencilOperation::IncrementClamp,
        0x06 => GpuStencilOperation::DecrementClamp,
        0x07 => GpuStencilOperation::IncrementWrap,
        0x08 => GpuStencilOperation::DecrementWrap,
        _ => panic!("Invalid stencil operation: {stencil_operation}"),
    }
}

fn enum_wgpu_storage_texture_access(access: WGPUStorageTextureAccess) -> GpuStorageTextureAccess {
    match access {
        0x00 => GpuStorageTextureAccess::__Invalid, // BindingNotUsed
        0x01 => GpuStorageTextureAccess::__Invalid, // Undefined
        0x02 => GpuStorageTextureAccess::WriteOnly,
        0x03 => GpuStorageTextureAccess::ReadOnly,
        0x04 => GpuStorageTextureAccess::ReadWrite,
        _ => panic!("Invalid storage texture access: {access}"),
    }
}

fn enum_wgpu_store_op(store_op: WGPUStoreOp) -> GpuStoreOp {
    match store_op {
        0x00 => GpuStoreOp::__Invalid, // Undefined
        0x01 => GpuStoreOp::Store,
        0x02 => GpuStoreOp::Discard,
        _ => panic!("Invalid store op: {store_op}"),
    }
}

fn enum_wgpu_texture_aspect(aspect: WGPUTextureAspect) -> GpuTextureAspect {
    match aspect {
        0x00 => GpuTextureAspect::__Invalid, // Undefined
        0x01 => GpuTextureAspect::All,
        0x02 => GpuTextureAspect::StencilOnly,
        0x03 => GpuTextureAspect::DepthOnly,
        _ => panic!("Invalid texture aspect: {aspect}"),
    }
}

fn enum_wgpu_texture_dimension(dimension: WGPUTextureDimension) -> GpuTextureDimension {
    match dimension {
        0x00 => GpuTextureDimension::__Invalid, // Undefined
        0x01 => GpuTextureDimension::N1d,
        0x02 => GpuTextureDimension::N2d,
        0x03 => GpuTextureDimension::N3d,
        _ => panic!("Invalid texture dimension: {dimension}"),
    }
}

macro_rules! define_texture_format_conversions {
    (
        $(
            $hex:literal => $gpu_variant:ident
        ),* $(,)?
    ) => {
        fn enum_wgpu_texture_format(format: WGPUTextureFormat) -> GpuTextureFormat {
            match format {
                $(
                    $hex => GpuTextureFormat::$gpu_variant,
                )*
                _ => panic!("Invalid texture format: {format}"),
            }
        }

        fn enum_wgpu_texture_format_from_gpu(format: GpuTextureFormat) -> WGPUTextureFormat {
            match format {
                $(
                    GpuTextureFormat::$gpu_variant => $hex,
                )*
                _ => panic!("Invalid texture format: {:?}", format),
            }
        }
    };
}

define_texture_format_conversions! {
    0x00 => __Invalid,
    0x01 => R8unorm,
    0x02 => R8snorm,
    0x03 => R8uint,
    0x04 => R8sint,
    0x05 => R16uint,
    0x06 => R16sint,
    0x07 => R16float,
    0x08 => Rg8unorm,
    0x09 => Rg8snorm,
    0x0A => Rg8uint,
    0x0B => Rg8sint,
    0x0C => R32float,
    0x0D => R32uint,
    0x0E => R32sint,
    0x0F => Rg16uint,
    0x10 => Rg16sint,
    0x11 => Rg16float,
    0x12 => Rgba8unorm,
    0x13 => Rgba8unormSrgb,
    0x14 => Rgba8snorm,
    0x15 => Rgba8uint,
    0x16 => Rgba8sint,
    0x17 => Bgra8unorm,
    0x18 => Bgra8unormSrgb,
    0x19 => Rgb10a2uint,
    0x1A => Rgb10a2unorm,
    0x1B => Rg11b10ufloat,
    0x1C => Rgb9e5ufloat,
    0x1D => Rg32float,
    0x1E => Rg32uint,
    0x1F => Rg32sint,
    0x20 => Rgba16uint,
    0x21 => Rgba16sint,
    0x22 => Rgba16float,
    0x23 => Rgba32float,
    0x24 => Rgba32uint,
    0x25 => Rgba32sint,
    0x26 => Stencil8,
    0x27 => Depth16unorm,
    0x28 => Depth24plus,
    0x29 => Depth24plusStencil8,
    0x2A => Depth32float,
    0x2B => Depth32floatStencil8,
    0x2C => Bc1RgbaUnorm,
    0x2D => Bc1RgbaUnormSrgb,
    0x2E => Bc2RgbaUnorm,
    0x2F => Bc2RgbaUnormSrgb,
    0x30 => Bc3RgbaUnorm,
    0x31 => Bc3RgbaUnormSrgb,
    0x32 => Bc4RUnorm,
    0x33 => Bc4RSnorm,
    0x34 => Bc5RgUnorm,
    0x35 => Bc5RgSnorm,
    0x36 => Bc6hRgbUfloat,
    0x37 => Bc6hRgbFloat,
    0x38 => Bc7RgbaUnorm,
    0x39 => Bc7RgbaUnormSrgb,
    0x3A => Etc2Rgb8unorm,
    0x3B => Etc2Rgb8unormSrgb,
    0x3C => Etc2Rgb8a1unorm,
    0x3D => Etc2Rgb8a1unormSrgb,
    0x3E => Etc2Rgba8unorm,
    0x3F => Etc2Rgba8unormSrgb,
    0x40 => EacR11unorm,
    0x41 => EacR11snorm,
    0x42 => EacRg11unorm,
    0x43 => EacRg11snorm,
    0x44 => Astc4x4Unorm,
    0x45 => Astc4x4UnormSrgb,
    0x46 => Astc5x4Unorm,
    0x47 => Astc5x4UnormSrgb,
    0x48 => Astc5x5Unorm,
    0x49 => Astc5x5UnormSrgb,
    0x4A => Astc6x5Unorm,
    0x4B => Astc6x5UnormSrgb,
    0x4C => Astc6x6Unorm,
    0x4D => Astc6x6UnormSrgb,
    0x4E => Astc8x5Unorm,
    0x4F => Astc8x5UnormSrgb,
    0x50 => Astc8x6Unorm,
    0x51 => Astc8x6UnormSrgb,
    0x52 => Astc8x8Unorm,
    0x53 => Astc8x8UnormSrgb,
    0x54 => Astc10x5Unorm,
    0x55 => Astc10x5UnormSrgb,
    0x56 => Astc10x6Unorm,
    0x57 => Astc10x6UnormSrgb,
    0x58 => Astc10x8Unorm,
    0x59 => Astc10x8UnormSrgb,
    0x5A => Astc10x10Unorm,
    0x5B => Astc10x10UnormSrgb,
    0x5C => Astc12x10Unorm,
    0x5D => Astc12x10UnormSrgb,
    0x5E => Astc12x12Unorm,
    0x5F => Astc12x12UnormSrgb,
}

fn enum_wgpu_texture_sample_type(sample_type: WGPUTextureSampleType) -> GpuTextureSampleType {
    match sample_type {
        0x00 => GpuTextureSampleType::__Invalid, // BindingNotUsed
        0x01 => GpuTextureSampleType::__Invalid, // Undefined
        0x02 => GpuTextureSampleType::Float,
        0x03 => GpuTextureSampleType::UnfilterableFloat,
        0x04 => GpuTextureSampleType::Depth,
        0x05 => GpuTextureSampleType::Sint,
        0x06 => GpuTextureSampleType::Uint,
        _ => panic!("Invalid texture sample type: {sample_type}"),
    }
}

fn enum_wgpu_texture_view_dimension(
    view_dimension: WGPUTextureViewDimension,
) -> GpuTextureViewDimension {
    match view_dimension {
        0x00 => GpuTextureViewDimension::__Invalid, // Undefined
        0x01 => GpuTextureViewDimension::N1d,
        0x02 => GpuTextureViewDimension::N2d,
        0x03 => GpuTextureViewDimension::N2dArray,
        0x04 => GpuTextureViewDimension::Cube,
        0x05 => GpuTextureViewDimension::CubeArray,
        0x06 => GpuTextureViewDimension::N3d,
        _ => panic!("Invalid texture view dimension: {view_dimension}"),
    }
}

fn enum_wgpu_vertex_format(format: WGPUVertexFormat) -> GpuVertexFormat {
    match format {
        0x00 => GpuVertexFormat::__Invalid,
        0x01 => GpuVertexFormat::Uint8,
        0x02 => GpuVertexFormat::Uint8x2,
        0x03 => GpuVertexFormat::Uint8x4,
        0x04 => GpuVertexFormat::Sint8,
        0x05 => GpuVertexFormat::Sint8x2,
        0x06 => GpuVertexFormat::Sint8x4,
        0x07 => GpuVertexFormat::Unorm8,
        0x08 => GpuVertexFormat::Unorm8x2,
        0x09 => GpuVertexFormat::Unorm8x4,
        0x0A => GpuVertexFormat::Snorm8,
        0x0B => GpuVertexFormat::Snorm8x2,
        0x0C => GpuVertexFormat::Snorm8x4,
        0x0D => GpuVertexFormat::Uint16,
        0x0E => GpuVertexFormat::Uint16x2,
        0x0F => GpuVertexFormat::Uint16x4,
        0x10 => GpuVertexFormat::Sint16,
        0x11 => GpuVertexFormat::Sint16x2,
        0x12 => GpuVertexFormat::Sint16x4,
        0x13 => GpuVertexFormat::Unorm16,
        0x14 => GpuVertexFormat::Unorm16x2,
        0x15 => GpuVertexFormat::Unorm16x4,
        0x16 => GpuVertexFormat::Snorm16,
        0x17 => GpuVertexFormat::Snorm16x2,
        0x18 => GpuVertexFormat::Snorm16x4,
        0x19 => GpuVertexFormat::Float16,
        0x1A => GpuVertexFormat::Float16x2,
        0x1B => GpuVertexFormat::Float16x4,
        0x1C => GpuVertexFormat::Float32,
        0x1D => GpuVertexFormat::Float32x2,
        0x1E => GpuVertexFormat::Float32x3,
        0x1F => GpuVertexFormat::Float32x4,
        0x20 => GpuVertexFormat::Uint32,
        0x21 => GpuVertexFormat::Uint32x2,
        0x22 => GpuVertexFormat::Uint32x3,
        0x23 => GpuVertexFormat::Uint32x4,
        0x24 => GpuVertexFormat::Sint32,
        0x25 => GpuVertexFormat::Sint32x2,
        0x26 => GpuVertexFormat::Sint32x3,
        0x27 => GpuVertexFormat::Sint32x4,
        0x28 => GpuVertexFormat::Unorm1010102,
        0x29 => GpuVertexFormat::Unorm8x4Bgra,
        _ => panic!("Invalid vertex format: {format}"),
    }
}

fn enum_wgpu_vertex_step_mode(step_mode: WGPUVertexStepMode) -> GpuVertexStepMode {
    match step_mode {
        0x00 => GpuVertexStepMode::__Invalid, // VertexBufferNotUsed
        0x01 => GpuVertexStepMode::__Invalid, // Undefined
        0x02 => GpuVertexStepMode::Vertex,
        0x03 => GpuVertexStepMode::Instance,
        _ => panic!("Invalid vertex step mode: {step_mode}"),
    }
}

// -------------------- Structs --------------------

#[repr(C)]
struct WGPUChainedStruct {
    next: *const WGPUChainedStruct,
    s_type: WGPUSType,
}

#[repr(C)]
struct WGPUBindGroupEntry {
    next_in_chain: *const WGPUChainedStruct,
    binding: u32,
    buffer: Option<NonNull<GpuBuffer>>,
    offset: u64,
    size: u64,
    sampler: Option<NonNull<GpuSampler>>,
    texture_view: Option<NonNull<GpuTextureView>>,
}

#[repr(C)]
struct WGPUBlendComponent {
    operation: WGPUBlendOperation,
    src_factor: WGPUBlendFactor,
    dst_factor: WGPUBlendFactor,
}

#[repr(C)]
struct WGPUBufferBindingLayout {
    next_in_chain: *const WGPUChainedStruct,
    type_: WGPUBufferBindingType,
    has_dynamic_offset: WGPUBool,
    min_binding_size: u64,
}

#[repr(C)]
struct WGPUBufferDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    usage: WGPUBufferUsage,
    size: u64,
    mapped_at_creation: WGPUBool,
}

#[repr(C)]
struct WGPUColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

#[repr(C)]
struct WGPUCommandBufferDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
}

#[repr(C)]
struct WGPUCommandEncoderDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
}

#[repr(C)]
struct WGPUComputeState {
    next_in_chain: *const WGPUChainedStruct,
    module: *mut GpuShaderModule,
    entry_point: WGPUStringView,
    constant_count: usize,
    constants: *const WGPUConstantEntry,
}

#[repr(C)]
struct WGPUConstantEntry {
    next_in_chain: *const WGPUChainedStruct,
    key: WGPUStringView,
    value: f64,
}

#[repr(C)]
struct WGPUExtent3D {
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
}

#[repr(C)]
struct WGPUMultisampleState {
    next_in_chain: *const WGPUChainedStruct,
    count: u32,
    mask: u32,
    alpha_to_coverage_enabled: WGPUBool,
}

#[repr(C)]
struct WGPUOrigin3D {
    x: u32,
    y: u32,
    z: u32,
}

#[repr(C)]
struct WGPUPassTimestampWrites {
    query_set: *mut GpuQuerySet,
    beginning_of_pass_write_index: u32,
    end_of_pass_write_index: u32,
}

#[repr(C)]
struct WGPUPipelineLayoutDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    bind_group_layout_count: usize,
    bind_group_layouts: *const *mut GpuBindGroupLayout,
}

#[repr(C)]
struct WGPUPrimitiveState {
    next_in_chain: *const WGPUChainedStruct,
    topology: WGPUPrimitiveTopology,
    strip_index_format: WGPUIndexFormat,
    front_face: WGPUFrontFace,
    cull_mode: WGPUCullMode,
    unclipped_depth: WGPUBool,
}

#[repr(C)]
struct WGPURenderPassDepthStencilAttachment {
    view: *mut GpuTextureView,
    depth_load_op: WGPULoadOp,
    depth_store_op: WGPUStoreOp,
    depth_clear_value: f32,
    depth_read_only: WGPUBool,
    stencil_load_op: WGPULoadOp,
    stencil_store_op: WGPUStoreOp,
    stencil_clear_value: u32,
    stencil_read_only: WGPUBool,
}

#[repr(C)]
struct WGPURenderPassTimestampWrites {
    query_set: *mut GpuQuerySet,
    beginning_of_pass_write_index: u32,
    end_of_pass_write_index: u32,
}

#[repr(C)]
struct WGPUSamplerBindingLayout {
    next_in_chain: *const WGPUChainedStruct,
    type_: WGPUSamplerBindingType,
}

#[repr(C)]
struct WGPUSamplerDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    address_mode_u: WGPUAddressMode,
    address_mode_v: WGPUAddressMode,
    address_mode_w: WGPUAddressMode,
    mag_filter: WGPUFilterMode,
    min_filter: WGPUFilterMode,
    mipmap_filter: WGPUMipmapFilterMode,
    lod_min_clamp: f32,
    lod_max_clamp: f32,
    compare: WGPUCompareFunction,
    max_anisotropy: u16,
}

#[repr(C)]
struct WGPUShaderModuleDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
}

#[repr(C)]
struct WGPUShaderSourceWGSL {
    chain: WGPUChainedStruct,
    code: WGPUStringView,
}

#[repr(C)]
struct WGPUStencilFaceState {
    compare: WGPUCompareFunction,
    fail_op: WGPUStencilOperation,
    depth_fail_op: WGPUStencilOperation,
    pass_op: WGPUStencilOperation,
}

#[repr(C)]
struct WGPUStorageTextureBindingLayout {
    next_in_chain: *const WGPUChainedStruct,
    access: WGPUStorageTextureAccess,
    format: WGPUTextureFormat,
    view_dimension: WGPUTextureViewDimension,
}

#[repr(C)]
struct WGPUStringView {
    data: Option<NonNull<c_char>>,
    length: usize,
}

#[repr(C)]
struct WGPUSurfaceTexture {
    next_in_chain: *const WGPUChainedStruct,
    texture: *mut GpuTexture,
    status: WGPUSurfaceGetCurrentTextureStatus,
}

#[repr(C)]
struct WGPUTextureBindingLayout {
    next_in_chain: *const WGPUChainedStruct,
    sample_type: WGPUTextureSampleType,
    view_dimension: WGPUTextureViewDimension,
    multisampled: WGPUBool,
}

#[repr(C)]
struct WGPUTextureDataLayout {
    offset: u64,
    bytes_per_row: u32,
    rows_per_image: u32,
}

#[repr(C)]
struct WGPUTextureViewDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    format: WGPUTextureFormat,
    dimension: WGPUTextureViewDimension,
    base_mip_level: u32,
    mip_level_count: u32,
    base_array_layer: u32,
    array_layer_count: u32,
    aspect: WGPUTextureAspect,
    usage: WGPUTextureUsage,
}

#[repr(C)]
struct WGPUVertexAttribute {
    format: WGPUVertexFormat,
    offset: u64,
    shader_location: u32,
}

#[repr(C)]
struct WGPUBindGroupDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    layout: *mut GpuBindGroupLayout,
    entry_count: usize,
    entries: *const WGPUBindGroupEntry,
}

#[repr(C)]
struct WGPUBindGroupLayoutEntry {
    next_in_chain: *const WGPUChainedStruct,
    binding: u32,
    visibility: WGPUShaderStage,
    buffer: WGPUBufferBindingLayout,
    sampler: WGPUSamplerBindingLayout,
    texture: WGPUTextureBindingLayout,
    storage_texture: WGPUStorageTextureBindingLayout,
}

#[repr(C)]
struct WGPUBlendState {
    color: WGPUBlendComponent,
    alpha: WGPUBlendComponent,
}

#[repr(C)]
struct WGPUComputePassDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    timestamp_writes: Option<NonNull<WGPUPassTimestampWrites>>,
}

#[repr(C)]
struct WGPUDepthStencilState {
    next_in_chain: *const WGPUChainedStruct,
    format: WGPUTextureFormat,
    depth_write_enabled: WGPUOptionalBool,
    depth_compare: WGPUCompareFunction,
    stencil_front: WGPUStencilFaceState,
    stencil_back: WGPUStencilFaceState,
    stencil_read_mask: u32,
    stencil_write_mask: u32,
    depth_bias: i32,
    depth_bias_slope_scale: f32,
    depth_bias_clamp: f32,
}

#[repr(C)]
struct WGPUImageCopyTexture {
    texture: *mut GpuTexture,
    mip_level: u32,
    origin: WGPUOrigin3D,
    aspect: WGPUTextureAspect,
}

#[repr(C)]
struct WGPURenderPassColorAttachment {
    next_in_chain: *const WGPUChainedStruct,
    view: Option<NonNull<GpuTextureView>>,
    depth_slice: u32,
    resolve_target: Option<NonNull<GpuTextureView>>,
    load_op: WGPULoadOp,
    store_op: WGPUStoreOp,
    clear_value: WGPUColor,
}

#[repr(C)]
struct WGPUTextureDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    usage: WGPUTextureUsage,
    dimension: WGPUTextureDimension,
    size: WGPUExtent3D,
    format: WGPUTextureFormat,
    mip_level_count: u32,
    sample_count: u32,
    view_format_count: usize,
    view_formats: *const WGPUTextureFormat,
}

#[repr(C)]
struct WGPUVertexBufferLayout {
    step_mode: WGPUVertexStepMode,
    array_stride: u64,
    attribute_count: usize,
    attributes: *const WGPUVertexAttribute,
}

#[repr(C)]
struct WGPUBindGroupLayoutDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    entry_count: usize,
    entries: *const WGPUBindGroupLayoutEntry,
}

#[repr(C)]
struct WGPUColorTargetState {
    next_in_chain: *const WGPUChainedStruct,
    format: WGPUTextureFormat,
    blend: Option<NonNull<WGPUBlendState>>,
    write_mask: WGPUColorWriteMask,
}

#[repr(C)]
struct WGPUComputePipelineDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    layout: Option<NonNull<GpuPipelineLayout>>,
    compute: WGPUComputeState,
}

#[repr(C)]
struct WGPURenderPassDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    color_attachment_count: usize,
    color_attachments: *const WGPURenderPassColorAttachment,
    depth_stencil_attachment: Option<NonNull<WGPURenderPassDepthStencilAttachment>>,
    occlusion_query_set: Option<NonNull<GpuQuerySet>>,
    timestamp_writes: Option<NonNull<WGPURenderPassTimestampWrites>>,
}

#[repr(C)]
struct WGPUVertexState {
    next_in_chain: *const WGPUChainedStruct,
    module: *mut GpuShaderModule,
    entry_point: WGPUStringView,
    constant_count: usize,
    constants: *const WGPUConstantEntry,
    buffer_count: usize,
    buffers: *const WGPUVertexBufferLayout,
}

#[repr(C)]
struct WGPUFragmentState {
    next_in_chain: *const WGPUChainedStruct,
    module: *mut GpuShaderModule,
    entry_point: WGPUStringView,
    constant_count: usize,
    constants: *const WGPUConstantEntry,
    target_count: usize,
    targets: *const WGPUColorTargetState,
}

#[repr(C)]
struct WGPURenderPipelineDescriptor {
    next_in_chain: *const WGPUChainedStruct,
    label: WGPUStringView,
    layout: Option<NonNull<GpuPipelineLayout>>,
    vertex: WGPUVertexState,
    primitive: WGPUPrimitiveState,
    depth_stencil: Option<NonNull<WGPUDepthStencilState>>,
    multisample: WGPUMultisampleState,
    fragment: Option<NonNull<WGPUFragmentState>>,
}

// -------------------- Convert webgpu.h structs to web_sys structs --------------------

fn set_descriptor_label(descriptor: &js_sys::Object, label: &WGPUStringView) {
    if let Some(label_str) = convert_wgpu_string_view(label) {
        js_sys::Reflect::set(descriptor, &"label".into(), &label_str.into()).unwrap();
    }
}

fn convert_wgpu_buffer_binding(buffer: &GpuBuffer, offset: u64, size: u64) -> GpuBufferBinding {
    let out = GpuBufferBinding::new(buffer);
    out.set_offset(offset as f64);
    out.set_size(size as f64);
    out
}

fn convert_wgpu_bind_group_entry(entry: &WGPUBindGroupEntry) -> GpuBindGroupEntry {
    if let Some(buffer) = entry.buffer {
        GpuBindGroupEntry::new(
            entry.binding,
            &convert_wgpu_buffer_binding(unsafe { &*buffer.as_ptr() }, entry.offset, entry.size),
        )
    } else if let Some(sampler) = entry.sampler {
        GpuBindGroupEntry::new(entry.binding, unsafe { &*sampler.as_ptr() })
    } else if let Some(texture_view) = entry.texture_view {
        GpuBindGroupEntry::new(entry.binding, unsafe { &*texture_view.as_ptr() })
    } else {
        panic!("No available type for binding group entry");
    }
}

fn convert_wgpu_blend_component(component: &WGPUBlendComponent) -> GpuBlendComponent {
    let out = GpuBlendComponent::new();
    out.set_operation(enum_wgpu_blend_operation(component.operation));
    out.set_src_factor(enum_wgpu_blend_factor(component.src_factor));
    out.set_dst_factor(enum_wgpu_blend_factor(component.dst_factor));
    out
}

fn convert_wgpu_buffer_binding_layout(layout: &WGPUBufferBindingLayout) -> GpuBufferBindingLayout {
    let out = GpuBufferBindingLayout::new();
    out.set_type(enum_wgpu_buffer_binding_type(layout.type_));
    out.set_has_dynamic_offset(layout.has_dynamic_offset != 0);
    out.set_min_binding_size(layout.min_binding_size as f64);
    out
}

fn convert_wgpu_buffer_descriptor(descriptor: &WGPUBufferDescriptor) -> GpuBufferDescriptor {
    let out = GpuBufferDescriptor::new(descriptor.size as f64, descriptor.usage as u32);
    set_descriptor_label(&out, &&descriptor.label);
    out.set_mapped_at_creation(descriptor.mapped_at_creation != 0);

    out
}

fn convert_wgpu_color(color: &WGPUColor) -> GpuColorDict {
    GpuColorDict::new(color.a, color.b, color.g, color.r)
}

fn convert_wgpu_command_buffer_descriptor(
    descriptor: &WGPUCommandBufferDescriptor,
) -> GpuCommandBufferDescriptor {
    let out = GpuCommandBufferDescriptor::new();
    set_descriptor_label(&out, &&descriptor.label);

    out
}

fn convert_wgpu_command_encoder_descriptor(
    descriptor: &WGPUCommandEncoderDescriptor,
) -> GpuCommandEncoderDescriptor {
    let out = GpuCommandEncoderDescriptor::new();
    set_descriptor_label(&out, &&descriptor.label);

    out
}

fn convert_wgpu_compute_state(descriptor: &WGPUComputeState) -> GpuProgrammableStage {
    let out = GpuProgrammableStage::new(unsafe { &*descriptor.module });
    if let Some(entry_point) = convert_wgpu_string_view(&descriptor.entry_point) {
        out.set_entry_point(&entry_point);
    }
    out.set_constants(&convert_wgpu_constants(
        descriptor.constants,
        descriptor.constant_count,
    ));
    out
}

fn convert_wgpu_constants(constants: *const WGPUConstantEntry, count: usize) -> js_sys::Map {
    let out = js_sys::Map::new();
    if constants.is_null() {
        return out;
    }

    let constants_slice = unsafe { std::slice::from_raw_parts(constants, count) };
    for constant in constants_slice {
        let key = convert_wgpu_string_view(&constant.key).expect("Could not convert key");
        out.set(&key.into(), &constant.value.into());
    }
    out
}

fn convert_wgpu_extent_3d(extent: &WGPUExtent3D) -> GpuExtent3dDict {
    let out = GpuExtent3dDict::new(extent.width);
    out.set_height(extent.height);
    out.set_depth_or_array_layers(extent.depth_or_array_layers);
    out
}

fn convert_wgpu_multisample_state(state: &WGPUMultisampleState) -> GpuMultisampleState {
    let out = GpuMultisampleState::new();
    out.set_count(state.count);
    out.set_mask(state.mask);
    out.set_alpha_to_coverage_enabled(state.alpha_to_coverage_enabled != 0);
    out
}

fn convert_wgpu_origin_3d(origin: &WGPUOrigin3D) -> js_sys::Array {
    let arr = js_sys::Array::new();
    arr.push(&origin.x.into());
    arr.push(&origin.y.into());
    arr.push(&origin.z.into());
    arr
}

fn convert_wgpu_pass_timestamp_writes(
    writes: &WGPUPassTimestampWrites,
) -> GpuComputePassTimestampWrites {
    let out = GpuComputePassTimestampWrites::new(unsafe { &*writes.query_set });
    out.set_beginning_of_pass_write_index(writes.beginning_of_pass_write_index);
    out.set_end_of_pass_write_index(writes.end_of_pass_write_index);
    out
}

fn convert_wgpu_pipeline_layout_descriptor(
    descriptor: &WGPUPipelineLayoutDescriptor,
) -> GpuPipelineLayoutDescriptor {
    // Create layouts array
    let layouts_array = js_sys::Array::new();
    if descriptor.bind_group_layout_count > 0 && !descriptor.bind_group_layouts.is_null() {
        let layouts = unsafe {
            std::slice::from_raw_parts(
                descriptor.bind_group_layouts,
                descriptor.bind_group_layout_count,
            )
        };

        for &layout in layouts {
            layouts_array.push(unsafe { &*layout });
        }
    }

    // Create descriptor
    let out = GpuPipelineLayoutDescriptor::new(&layouts_array.into());
    set_descriptor_label(&out, &descriptor.label);

    out
}

fn convert_wgpu_primitive_state(state: &WGPUPrimitiveState) -> GpuPrimitiveState {
    let out = GpuPrimitiveState::new();
    out.set_topology(enum_wgpu_primitive_topology(state.topology));
    out.set_strip_index_format(enum_wgpu_index_format(state.strip_index_format));
    out.set_front_face(enum_wgpu_front_face(state.front_face));
    out.set_cull_mode(enum_wgpu_cull_mode(state.cull_mode));
    out.set_unclipped_depth(state.unclipped_depth != 0);
    out
}

fn convert_wgpu_render_pass_depth_stencil_attachment(
    attachment: &WGPURenderPassDepthStencilAttachment,
) -> GpuRenderPassDepthStencilAttachment {
    let out = GpuRenderPassDepthStencilAttachment::new(unsafe { &*attachment.view });
    out.set_depth_load_op(enum_wgpu_load_op(attachment.depth_load_op));
    out.set_depth_store_op(enum_wgpu_store_op(attachment.depth_store_op));
    out.set_depth_clear_value(attachment.depth_clear_value);
    out.set_depth_read_only(attachment.depth_read_only != 0);
    out.set_stencil_load_op(enum_wgpu_load_op(attachment.stencil_load_op));
    out.set_stencil_store_op(enum_wgpu_store_op(attachment.stencil_store_op));
    out.set_stencil_clear_value(attachment.stencil_clear_value);
    out.set_stencil_read_only(attachment.stencil_read_only != 0);
    out
}

fn convert_wgpu_render_pass_timestamp_writes(
    writes: &WGPURenderPassTimestampWrites,
) -> GpuRenderPassTimestampWrites {
    let out = GpuRenderPassTimestampWrites::new(unsafe { &*writes.query_set });
    out.set_beginning_of_pass_write_index(writes.beginning_of_pass_write_index);
    out.set_end_of_pass_write_index(writes.end_of_pass_write_index);
    out
}

fn convert_wgpu_sampler_binding_layout(
    layout: &WGPUSamplerBindingLayout,
) -> GpuSamplerBindingLayout {
    let out = GpuSamplerBindingLayout::new();
    out.set_type(enum_wgpu_sampler_binding_type(layout.type_));
    out
}

fn convert_wgpu_sampler_descriptor(descriptor: &WGPUSamplerDescriptor) -> GpuSamplerDescriptor {
    let out = GpuSamplerDescriptor::new();
    set_descriptor_label(&out, &descriptor.label);

    out.set_address_mode_u(enum_wgpu_address_mode(descriptor.address_mode_u));
    out.set_address_mode_v(enum_wgpu_address_mode(descriptor.address_mode_v));
    out.set_address_mode_w(enum_wgpu_address_mode(descriptor.address_mode_w));
    out.set_mag_filter(enum_wgpu_filter_mode(descriptor.mag_filter));
    out.set_min_filter(enum_wgpu_filter_mode(descriptor.min_filter));
    out.set_mipmap_filter(enum_wgpu_mipmap_filter_mode(descriptor.mipmap_filter));
    out.set_lod_min_clamp(descriptor.lod_min_clamp);
    out.set_lod_max_clamp(descriptor.lod_max_clamp);
    out.set_compare(enum_wgpu_compare_function(descriptor.compare));
    out.set_max_anisotropy(descriptor.max_anisotropy);

    out
}

fn convert_wgpu_shader_source_wgsl(descriptor: &WGPUShaderSourceWGSL) -> GpuShaderModuleDescriptor {
    let code = convert_wgpu_string_view(&descriptor.code).unwrap_or_default();
    GpuShaderModuleDescriptor::new(&code)
}

fn convert_wgpu_shader_module_descriptor(
    descriptor: &WGPUShaderModuleDescriptor,
) -> GpuShaderModuleDescriptor {
    // Get the chained WGSL descriptor
    let wgsl_desc = unsafe { &*(descriptor.next_in_chain as *const WGPUShaderSourceWGSL) };

    if wgsl_desc.chain.s_type == 2 {
        // WGPUSType_ShaderModuleWGSLDescriptor
        let out = convert_wgpu_shader_source_wgsl(wgsl_desc);
        set_descriptor_label(&out, &descriptor.label);

        out
    } else {
        panic!(
            "Unsupported shader module type: 0x{:X} (expected 0x00000002)",
            wgsl_desc.chain.s_type
        );
    }
}

fn convert_wgpu_stencil_face_state(state: &WGPUStencilFaceState) -> GpuStencilFaceState {
    let out = GpuStencilFaceState::new();
    out.set_compare(enum_wgpu_compare_function(state.compare));
    out.set_fail_op(enum_wgpu_stencil_operation(state.fail_op));
    out.set_depth_fail_op(enum_wgpu_stencil_operation(state.depth_fail_op));
    out.set_pass_op(enum_wgpu_stencil_operation(state.pass_op));
    out
}

fn convert_wgpu_storage_texture_binding_layout(
    layout: &WGPUStorageTextureBindingLayout,
) -> GpuStorageTextureBindingLayout {
    let out = GpuStorageTextureBindingLayout::new(enum_wgpu_texture_format(layout.format));
    out.set_access(enum_wgpu_storage_texture_access(layout.access));
    out.set_view_dimension(enum_wgpu_texture_view_dimension(layout.view_dimension));
    out
}

fn convert_wgpu_string_view(string_view: &WGPUStringView) -> Option<String> {
    if string_view.length == 0 {
        return None;
    }

    let string = string_view
        .data
        .map(|data| unsafe { CStr::from_ptr(data.as_ptr()).to_str().unwrap().into() });

    return string;
}

fn convert_wgpu_texture_binding_layout(
    layout: &WGPUTextureBindingLayout,
) -> GpuTextureBindingLayout {
    let out = GpuTextureBindingLayout::new();
    out.set_sample_type(enum_wgpu_texture_sample_type(layout.sample_type));
    out.set_view_dimension(enum_wgpu_texture_view_dimension(layout.view_dimension));
    out.set_multisampled(layout.multisampled != 0);
    out
}

fn convert_wgpu_texture_data_layout(layout: &WGPUTextureDataLayout) -> GpuTexelCopyBufferLayout {
    let out = GpuTexelCopyBufferLayout::new();
    out.set_offset(layout.offset as f64);
    out.set_bytes_per_row(layout.bytes_per_row);
    out.set_rows_per_image(layout.rows_per_image);
    out
}

fn convert_wgpu_texture_view_descriptor(
    descriptor: &WGPUTextureViewDescriptor,
) -> GpuTextureViewDescriptor {
    let out = GpuTextureViewDescriptor::new();
    set_descriptor_label(&out, &descriptor.label);

    out.set_format(enum_wgpu_texture_format(descriptor.format));
    out.set_dimension(enum_wgpu_texture_view_dimension(descriptor.dimension));
    out.set_base_mip_level(descriptor.base_mip_level);
    out.set_mip_level_count(descriptor.mip_level_count);
    out.set_base_array_layer(descriptor.base_array_layer);
    out.set_array_layer_count(descriptor.array_layer_count);
    out.set_aspect(enum_wgpu_texture_aspect(descriptor.aspect));

    out
}

fn convert_wgpu_vertex_attribute(attribute: &WGPUVertexAttribute) -> GpuVertexAttribute {
    GpuVertexAttribute::new(
        enum_wgpu_vertex_format(attribute.format),
        attribute.offset as f64,
        attribute.shader_location,
    )
}

fn convert_wgpu_bind_group_descriptor(
    descriptor: &WGPUBindGroupDescriptor,
) -> GpuBindGroupDescriptor {
    // Create entries array
    let entries_array = js_sys::Array::new();
    let entries_slice =
        unsafe { std::slice::from_raw_parts(descriptor.entries, descriptor.entry_count) };

    for entry in entries_slice {
        let converted_entry = convert_wgpu_bind_group_entry(entry);
        entries_array.push(&converted_entry);
    }

    // Create descriptor
    let out = GpuBindGroupDescriptor::new(&entries_array.into(), unsafe { &*descriptor.layout });
    set_descriptor_label(&out, &descriptor.label);

    out
}

fn convert_wgpu_bind_group_layout_entry(
    entry: &WGPUBindGroupLayoutEntry,
) -> GpuBindGroupLayoutEntry {
    let out = GpuBindGroupLayoutEntry::new(entry.binding, entry.visibility as u32);

    if entry.buffer.type_ != 0 {
        let buffer_binding = convert_wgpu_buffer_binding_layout(&entry.buffer);
        out.set_buffer(&buffer_binding);
    } else if entry.sampler.type_ != 0 {
        let sampler_binding = convert_wgpu_sampler_binding_layout(&entry.sampler);
        out.set_sampler(&sampler_binding);
    } else if entry.texture.sample_type != 0 {
        let texture_binding = convert_wgpu_texture_binding_layout(&entry.texture);
        out.set_texture(&texture_binding);
    } else if entry.storage_texture.access != 0 {
        let storage_binding = convert_wgpu_storage_texture_binding_layout(&entry.storage_texture);
        out.set_storage_texture(&storage_binding);
    } else {
        panic!("Unknown bind group layout entry type");
    }

    out
}

fn convert_wgpu_blend_state(state: &WGPUBlendState) -> GpuBlendState {
    GpuBlendState::new(
        &convert_wgpu_blend_component(&state.color),
        &convert_wgpu_blend_component(&state.alpha),
    )
}

fn convert_wgpu_compute_pass_descriptor(
    descriptor: &WGPUComputePassDescriptor,
) -> GpuComputePassDescriptor {
    let out = GpuComputePassDescriptor::new();
    set_descriptor_label(&out, &descriptor.label);
    if let Some(timestamp_writes) = descriptor.timestamp_writes {
        let timestamp_writes =
            convert_wgpu_pass_timestamp_writes(unsafe { &*timestamp_writes.as_ptr() });
        out.set_timestamp_writes(&timestamp_writes);
    }

    out
}

fn convert_wgpu_depth_stencil_state(state: &WGPUDepthStencilState) -> GpuDepthStencilState {
    let out = GpuDepthStencilState::new(enum_wgpu_texture_format(state.format));
    if let Some(enabled) = enum_wgpu_optional_bool(state.depth_write_enabled) {
        out.set_depth_write_enabled(enabled);
    }
    out.set_depth_compare(enum_wgpu_compare_function(state.depth_compare));
    out.set_stencil_front(&convert_wgpu_stencil_face_state(&state.stencil_front));
    out.set_stencil_back(&convert_wgpu_stencil_face_state(&state.stencil_back));
    out.set_stencil_read_mask(state.stencil_read_mask);
    out.set_stencil_write_mask(state.stencil_write_mask);
    out.set_depth_bias(state.depth_bias);
    out.set_depth_bias_slope_scale(state.depth_bias_slope_scale);
    out.set_depth_bias_clamp(state.depth_bias_clamp);
    out
}

fn convert_wgpu_image_copy_texture(texture: &WGPUImageCopyTexture) -> GpuTexelCopyTextureInfo {
    let out = GpuTexelCopyTextureInfo::new(unsafe { &*texture.texture });
    out.set_mip_level(texture.mip_level);
    out.set_origin(&convert_wgpu_origin_3d(&texture.origin));
    out.set_aspect(enum_wgpu_texture_aspect(texture.aspect));
    out
}

fn convert_wgpu_render_pass_color_attachment(
    attachment: &WGPURenderPassColorAttachment,
) -> GpuRenderPassColorAttachment {
    let view = if let Some(view) = attachment.view {
        unsafe { &*view.as_ptr() }
    } else {
        panic!("No texture view found");
    };

    let out = GpuRenderPassColorAttachment::new(
        enum_wgpu_load_op(attachment.load_op),
        enum_wgpu_store_op(attachment.store_op),
        view,
    );
    out.set_clear_value(&convert_wgpu_color(&attachment.clear_value));

    // Set depth slice if not max value
    if attachment.depth_slice != u32::MAX {
        out.set_depth_slice(attachment.depth_slice);
    }

    // Set resolve target if present
    if let Some(resolve_target) = attachment.resolve_target {
        out.set_resolve_target(unsafe { &*resolve_target.as_ptr() });
    }

    out
}

fn convert_wgpu_texture_descriptor(descriptor: &WGPUTextureDescriptor) -> GpuTextureDescriptor {
    // Create descriptor
    let out = GpuTextureDescriptor::new(
        enum_wgpu_texture_format(descriptor.format),
        &convert_wgpu_extent_3d(&descriptor.size),
        descriptor.usage as u32,
    );
    set_descriptor_label(&out, &descriptor.label);

    // Set other properties
    out.set_mip_level_count(descriptor.mip_level_count);
    out.set_sample_count(descriptor.sample_count);
    out.set_dimension(enum_wgpu_texture_dimension(descriptor.dimension));

    // Set view formats if present
    if !descriptor.view_formats.is_null() {
        let view_formats = unsafe {
            std::slice::from_raw_parts(descriptor.view_formats, descriptor.view_format_count)
        };
        let formats_array = js_sys::Array::new();
        for &format in view_formats {
            formats_array.push(&format.into());
        }
        out.set_view_formats(&formats_array.into());
    }

    out
}

fn convert_wgpu_vertex_buffer_layout(layout: &WGPUVertexBufferLayout) -> GpuVertexBufferLayout {
    // Create attributes array
    let attributes = js_sys::Array::new();
    let attributes_slice =
        unsafe { std::slice::from_raw_parts(layout.attributes, layout.attribute_count) };
    for attribute in attributes_slice {
        attributes.push(&convert_wgpu_vertex_attribute(attribute));
    }

    // Create layout
    let out = GpuVertexBufferLayout::new(layout.array_stride as f64, &attributes.into());
    out.set_step_mode(enum_wgpu_vertex_step_mode(layout.step_mode));
    out
}

fn convert_wgpu_bind_group_layout_descriptor(
    descriptor: &WGPUBindGroupLayoutDescriptor,
) -> GpuBindGroupLayoutDescriptor {
    // Create entries array
    let entries = js_sys::Array::new();
    let entries_slice =
        unsafe { std::slice::from_raw_parts(descriptor.entries, descriptor.entry_count) };

    for entry in entries_slice {
        let converted_entry = convert_wgpu_bind_group_layout_entry(entry);
        entries.push(&converted_entry);
    }

    // Create descriptor
    let out = GpuBindGroupLayoutDescriptor::new(&entries.into());
    set_descriptor_label(&out, &descriptor.label);

    out
}

fn convert_wgpu_color_target_state(target: &WGPUColorTargetState) -> GpuColorTargetState {
    let out = GpuColorTargetState::new(enum_wgpu_texture_format(target.format));
    out.set_write_mask(target.write_mask as u32);
    if let Some(blend) = target.blend {
        out.set_blend(&convert_wgpu_blend_state(unsafe { &*blend.as_ptr() }));
    }
    out
}

fn convert_wgpu_compute_pipeline_descriptor(
    descriptor: &WGPUComputePipelineDescriptor,
) -> GpuComputePipelineDescriptor {
    let layout = if let Some(layout) = descriptor.layout {
        unsafe { &*layout.as_ptr() }
    } else {
        panic!("No layout found");
    };

    let out =
        GpuComputePipelineDescriptor::new(layout, &convert_wgpu_compute_state(&descriptor.compute));
    set_descriptor_label(&out, &descriptor.label);

    out
}

fn convert_wgpu_render_pass_descriptor(
    descriptor: &WGPURenderPassDescriptor,
) -> GpuRenderPassDescriptor {
    // Set color attachments if present
    let color_array = js_sys::Array::new();
    if descriptor.color_attachment_count > 0 && !descriptor.color_attachments.is_null() {
        let attachments_slice = unsafe {
            std::slice::from_raw_parts(
                descriptor.color_attachments,
                descriptor.color_attachment_count,
            )
        };

        for attachment in attachments_slice {
            let wgpu_attachment = convert_wgpu_render_pass_color_attachment(attachment);
            color_array.push(&wgpu_attachment);
        }
    }

    // Create descriptor
    let out = GpuRenderPassDescriptor::new(&color_array.into());
    set_descriptor_label(&out, &descriptor.label);

    // Set occlusion query set if present
    if let Some(occlusion_query_set) = descriptor.occlusion_query_set {
        out.set_occlusion_query_set(unsafe { &*occlusion_query_set.as_ptr() });
    }

    // Set timestamp writes if present
    if let Some(timestamp_writes) = descriptor.timestamp_writes {
        out.set_timestamp_writes(&convert_wgpu_render_pass_timestamp_writes(unsafe {
            &*timestamp_writes.as_ptr()
        }));
    }

    // Set depth stencil attachment if present
    if let Some(depth_stencil_attachment) = descriptor.depth_stencil_attachment {
        let depth_stencil = convert_wgpu_render_pass_depth_stencil_attachment(unsafe {
            &*depth_stencil_attachment.as_ptr()
        });
        out.set_depth_stencil_attachment(&depth_stencil);
    }

    out
}

fn convert_wgpu_vertex_state(state: &WGPUVertexState) -> GpuVertexState {
    let out = GpuVertexState::new(unsafe { &*state.module });
    out.set_constants(&convert_wgpu_constants(
        state.constants,
        state.constant_count,
    ));
    if let Some(entry_point) = convert_wgpu_string_view(&state.entry_point) {
        out.set_entry_point(&entry_point);
    }

    // Set vertex buffer layouts if present
    if !state.buffers.is_null() {
        let buffer_layouts = js_sys::Array::new();
        let buffers_slice =
            unsafe { std::slice::from_raw_parts(state.buffers, state.buffer_count) };

        for buffer in buffers_slice {
            let buffer_layout = convert_wgpu_vertex_buffer_layout(buffer);
            buffer_layouts.push(&buffer_layout);
        }
        out.set_buffers(&buffer_layouts.into());
    }

    out
}

fn convert_wgpu_fragment_state(state: &WGPUFragmentState) -> GpuFragmentState {
    // Create targets array
    let targets = js_sys::Array::new();
    let targets_slice = unsafe { std::slice::from_raw_parts(state.targets, state.target_count) };
    for target in targets_slice {
        let target_state = convert_wgpu_color_target_state(target);
        targets.push(&target_state);
    }

    // Create fragment state
    let out = GpuFragmentState::new(unsafe { &*state.module }, &targets.into());
    out.set_constants(&convert_wgpu_constants(
        state.constants,
        state.constant_count,
    ));
    if let Some(entry_point) = convert_wgpu_string_view(&state.entry_point) {
        out.set_entry_point(&entry_point);
    }
    out
}

fn convert_wgpu_render_pipeline_descriptor(
    descriptor: &WGPURenderPipelineDescriptor,
) -> GpuRenderPipelineDescriptor {
    let layout = if let Some(layout) = descriptor.layout {
        unsafe { &*layout.as_ptr() }
    } else {
        panic!("No layout found");
    };

    let out =
        GpuRenderPipelineDescriptor::new(layout, &convert_wgpu_vertex_state(&descriptor.vertex));
    set_descriptor_label(&out, &descriptor.label);

    out.set_primitive(&convert_wgpu_primitive_state(&descriptor.primitive));
    out.set_multisample(&convert_wgpu_multisample_state(&descriptor.multisample));
    if let Some(depth_stencil) = descriptor.depth_stencil {
        out.set_depth_stencil(&convert_wgpu_depth_stencil_state(unsafe {
            &*depth_stencil.as_ptr()
        }));
    }
    if let Some(fragment) = descriptor.fragment {
        out.set_fragment(&convert_wgpu_fragment_state(unsafe { &*fragment.as_ptr() }));
    }

    out
}

// -------------------- WebGPU API --------------------

#[no_mangle]
unsafe extern "C" fn wgpuCreateInstance(_descriptor: *const c_void) -> *mut c_void {
    // This function is now just a stub since the actual instance creation is handled in webgpu_init_async
    // This prevents errors when the C++ code tries to call this function
    std::ptr::null_mut()
}

// Methods of Adapter

#[no_mangle]
unsafe extern "C" fn wgpuAdapterRequestDevice(
    _adapter: *mut c_void,
    _descriptor: *const c_void,
) -> *mut c_void {
    // This function is now just a stub since the actual device request is handled in webgpu_init_async
    // This prevents errors when the C++ code tries to call this function
    std::ptr::null_mut()
}

#[no_mangle]
unsafe extern "C" fn wgpuAdapterRelease(adapter: *mut GpuAdapter) {
    drop(Box::from_raw(adapter));
}

// Methods of BindGroup

#[no_mangle]
unsafe extern "C" fn wgpuBindGroupRelease(bind_group: *mut GpuBindGroup) {
    drop(Box::from_raw(bind_group));
}

// Methods of BindGroupLayout

#[no_mangle]
unsafe extern "C" fn wgpuBindGroupLayoutRelease(layout: *mut GpuBindGroupLayout) {
    drop(Box::from_raw(layout));
}

// Methods of Buffer

#[no_mangle]
unsafe extern "C" fn wgpuBufferDestroy(buffer: *mut GpuBuffer) {
    let buffer = &*buffer;
    buffer.destroy();
}

#[no_mangle]
unsafe extern "C" fn wgpuBufferGetSize(buffer: *mut GpuBuffer) -> u64 {
    let buffer = &*buffer;
    buffer.size() as u64
}

#[no_mangle]
unsafe extern "C" fn wgpuBufferRelease(buffer: *mut GpuBuffer) {
    drop(Box::from_raw(buffer));
}

// Methods of CommandBuffer

#[no_mangle]
unsafe extern "C" fn wgpuCommandBufferRelease(buffer: *mut GpuCommandBuffer) {
    drop(Box::from_raw(buffer));
}

// Methods of CommandEncoder

#[no_mangle]
unsafe extern "C" fn wgpuCommandEncoderBeginComputePass(
    encoder: *mut GpuCommandEncoder,
    descriptor: *const WGPUComputePassDescriptor,
) -> *mut GpuComputePassEncoder {
    let encoder = &*encoder;
    let descriptor = convert_wgpu_compute_pass_descriptor(&*descriptor);
    let pass = encoder.begin_compute_pass_with_descriptor(&descriptor);
    Box::into_raw(Box::new(pass))
}

#[no_mangle]
unsafe extern "C" fn wgpuCommandEncoderBeginRenderPass(
    encoder: *mut GpuCommandEncoder,
    descriptor: *const WGPURenderPassDescriptor,
) -> *mut GpuRenderPassEncoder {
    let encoder = &*encoder;
    let descriptor = convert_wgpu_render_pass_descriptor(&*descriptor);
    let render_pass = encoder
        .begin_render_pass(&descriptor)
        .expect("Failed to begin render pass");
    Box::into_raw(Box::new(render_pass))
}

#[no_mangle]
unsafe extern "C" fn wgpuCommandEncoderCopyTextureToTexture(
    encoder: *mut GpuCommandEncoder,
    source: *const WGPUImageCopyTexture,
    destination: *const WGPUImageCopyTexture,
    copy_size: *const WGPUExtent3D,
) {
    let encoder = &*encoder;
    let source = &convert_wgpu_image_copy_texture(&*source);
    let destination = &convert_wgpu_image_copy_texture(&*destination);
    let copy_size = &convert_wgpu_extent_3d(&*copy_size);
    match encoder.copy_texture_to_texture_with_gpu_extent_3d_dict(source, destination, copy_size) {
        Ok(_) => (),
        Err(e) => panic!("Failed to copy texture: {e:?}"),
    }
}

#[no_mangle]
unsafe extern "C" fn wgpuCommandEncoderFinish(
    encoder: *mut GpuCommandEncoder,
    descriptor: *const WGPUCommandBufferDescriptor,
) -> *mut GpuCommandBuffer {
    let encoder = &*encoder;
    if descriptor.is_null() {
        let command_buffer = encoder.finish();
        Box::into_raw(Box::new(command_buffer))
    } else {
        let descriptor = convert_wgpu_command_buffer_descriptor(&*descriptor);
        let command_buffer = encoder.finish_with_descriptor(&descriptor);
        Box::into_raw(Box::new(command_buffer))
    }
}

#[no_mangle]
unsafe extern "C" fn wgpuCommandEncoderRelease(encoder: *mut GpuCommandEncoder) {
    drop(Box::from_raw(encoder));
}

// Methods of ComputePassEncoder

#[no_mangle]
unsafe extern "C" fn wgpuComputePassEncoderDispatchWorkgroups(
    pass: *mut GpuComputePassEncoder,
    workgroup_count_x: u32,
    workgroup_count_y: u32,
    workgroup_count_z: u32,
) {
    let pass = &*pass;
    pass.dispatch_workgroups_with_workgroup_count_y_and_workgroup_count_z(
        workgroup_count_x,
        workgroup_count_y,
        workgroup_count_z,
    );
}

#[no_mangle]
unsafe extern "C" fn wgpuComputePassEncoderEnd(pass: *mut GpuComputePassEncoder) {
    let pass = &*pass;
    pass.end();
}

#[no_mangle]
unsafe extern "C" fn wgpuComputePassEncoderSetBindGroup(
    pass: *mut GpuComputePassEncoder,
    index: u32,
    bind_group: Option<NonNull<GpuBindGroup>>,
    dynamic_offset_count: usize,
    dynamic_offsets: *const u32,
) {
    let pass = &*pass;
    let bind_group = bind_group.map(|b| b.as_ref());
    if dynamic_offset_count > 0 {
        let offsets = unsafe { std::slice::from_raw_parts(dynamic_offsets, dynamic_offset_count) };
        let js_offsets = js_sys::Array::new();
        for &offset in offsets {
            js_offsets.push(&JsValue::from(offset));
        }
        pass.set_bind_group_with_u32_sequence(index, bind_group, &js_offsets.into());
    } else {
        pass.set_bind_group(index, bind_group);
    }
}

#[no_mangle]
unsafe extern "C" fn wgpuComputePassEncoderSetPipeline(
    pass: *mut GpuComputePassEncoder,
    pipeline: *mut GpuComputePipeline,
) {
    let pass = &*pass;
    let pipeline = &*pipeline;
    pass.set_pipeline(pipeline);
}

#[no_mangle]
unsafe extern "C" fn wgpuComputePassEncoderRelease(pass: *mut GpuComputePassEncoder) {
    drop(Box::from_raw(pass));
}

// Methods of ComputePipeline

#[no_mangle]
unsafe extern "C" fn wgpuComputePipelineRelease(pipeline: *mut GpuComputePipeline) {
    drop(Box::from_raw(pipeline));
}

// Methods of Device

#[no_mangle]
unsafe extern "C" fn wgpuDeviceCreateBindGroup(
    device: *mut GpuDevice,
    descriptor: *const WGPUBindGroupDescriptor,
) -> *mut GpuBindGroup {
    let device = &*device;
    let descriptor = convert_wgpu_bind_group_descriptor(&*descriptor);
    let bind_group = device.create_bind_group(&descriptor);
    Box::into_raw(Box::new(bind_group))
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceCreateBindGroupLayout(
    device: *mut GpuDevice,
    descriptor: *const WGPUBindGroupLayoutDescriptor,
) -> *mut GpuBindGroupLayout {
    let device = &*device;
    let descriptor = convert_wgpu_bind_group_layout_descriptor(&*descriptor);
    let layout = device
        .create_bind_group_layout(&descriptor)
        .expect("Failed to create bind group layout");
    Box::into_raw(Box::new(layout))
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceCreateBuffer(
    device: *mut GpuDevice,
    descriptor: *const WGPUBufferDescriptor,
) -> *mut GpuBuffer {
    let device = &*device;
    let descriptor = convert_wgpu_buffer_descriptor(&*descriptor);
    let buffer = device
        .create_buffer(&descriptor)
        .expect("Failed to create buffer");
    Box::into_raw(Box::new(buffer))
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceCreateCommandEncoder(
    device: *mut GpuDevice,
    descriptor: *const WGPUCommandEncoderDescriptor,
) -> *mut GpuCommandEncoder {
    let device = &*device;
    let descriptor = convert_wgpu_command_encoder_descriptor(&*descriptor);
    let encoder = device.create_command_encoder_with_descriptor(&descriptor);
    Box::into_raw(Box::new(encoder))
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceCreateComputePipeline(
    device: *mut GpuDevice,
    descriptor: *const WGPUComputePipelineDescriptor,
) -> *mut GpuComputePipeline {
    let device = &*device;
    let descriptor = convert_wgpu_compute_pipeline_descriptor(&*descriptor);
    let pipeline = device.create_compute_pipeline(&descriptor);
    Box::into_raw(Box::new(pipeline))
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceCreatePipelineLayout(
    device: *mut GpuDevice,
    descriptor: *const WGPUPipelineLayoutDescriptor,
) -> *mut GpuPipelineLayout {
    let device = &*device;
    let descriptor = convert_wgpu_pipeline_layout_descriptor(&*descriptor);
    let layout = device.create_pipeline_layout(&descriptor);
    Box::into_raw(Box::new(layout))
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceCreateRenderPipeline(
    device: *mut GpuDevice,
    descriptor: *const WGPURenderPipelineDescriptor,
) -> *mut GpuRenderPipeline {
    let device = &*device;
    let descriptor = convert_wgpu_render_pipeline_descriptor(&*descriptor);
    let pipeline = device
        .create_render_pipeline(&descriptor)
        .expect("Failed to create render pipeline");
    Box::into_raw(Box::new(pipeline))
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceCreateSampler(
    device: *mut GpuDevice,
    descriptor: *const WGPUSamplerDescriptor,
) -> *mut GpuSampler {
    let device = &*device;
    let descriptor = convert_wgpu_sampler_descriptor(&*descriptor);
    let sampler = device.create_sampler_with_descriptor(&descriptor);
    Box::into_raw(Box::new(sampler))
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceCreateShaderModule(
    device: *mut GpuDevice,
    descriptor: *const WGPUShaderModuleDescriptor,
) -> *mut GpuShaderModule {
    let device = &*device;
    let descriptor = convert_wgpu_shader_module_descriptor(&*descriptor);
    let module = device.create_shader_module(&descriptor);
    Box::into_raw(Box::new(module))
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceCreateTexture(
    device: *mut GpuDevice,
    descriptor: *const WGPUTextureDescriptor,
) -> *mut GpuTexture {
    let device = &*device;
    let descriptor = convert_wgpu_texture_descriptor(&*descriptor);
    let texture = device
        .create_texture(&descriptor)
        .expect("Failed to create texture");
    Box::into_raw(Box::new(texture))
}

#[no_mangle]
unsafe extern "C" fn wgpuDeviceGetQueue(device: *mut GpuDevice) -> *mut GpuQueue {
    let device = &*device;
    let queue = device.queue();
    Box::into_raw(Box::new(queue))
}

// Methods of PipelineLayout

#[no_mangle]
unsafe extern "C" fn wgpuPipelineLayoutRelease(layout: *mut GpuPipelineLayout) {
    drop(Box::from_raw(layout));
}

// Methods of Queue

#[no_mangle]
unsafe extern "C" fn wgpuQueueSubmit(
    queue: *mut GpuQueue,
    command_count: usize,
    commands: *const *mut GpuCommandBuffer,
) {
    let queue = &*queue;

    // Handle zero command count case
    if command_count == 0 {
        queue.submit(&js_sys::Array::new());
        return;
    }

    // Create array to hold command buffers
    let cmd_array = js_sys::Array::new();
    // Convert each command buffer pointer to a JsValue and add to array
    let commands_slice = std::slice::from_raw_parts(commands, command_count);
    for &cmd in commands_slice {
        let command_buffer = &*cmd;
        cmd_array.push(command_buffer);
    }

    queue.submit(&cmd_array);
}

#[no_mangle]
unsafe extern "C" fn wgpuQueueWriteBuffer(
    queue: *mut GpuQueue,
    buffer: *mut GpuBuffer,
    buffer_offset: u64,
    data: *const u8,
    size: usize,
) {
    let queue = &*queue;
    let buffer = &*buffer;

    if size == 0 {
        let uint8_array = js_sys::Uint8Array::new_with_length(0);
        let _ = queue.write_buffer_with_f64_and_buffer_source(
            buffer,
            buffer_offset as f64,
            &uint8_array,
        );
        return;
    }

    // Create a copy of the data to ensure it stays valid
    let mut data_copy = Vec::with_capacity(size);
    data_copy.extend_from_slice(unsafe { std::slice::from_raw_parts(data, size) });

    // Create a typed array from the copied data
    let uint8_array = js_sys::Uint8Array::from(data_copy.as_slice());

    // Use the direct WebGPU API call
    let _ =
        queue.write_buffer_with_f64_and_buffer_source(buffer, buffer_offset as f64, &uint8_array);
}

#[no_mangle]
unsafe extern "C" fn wgpuQueueWriteTexture(
    queue: *mut GpuQueue,
    destination: *const WGPUImageCopyTexture,
    data: *const u8,
    data_size: usize,
    data_layout: *const WGPUTextureDataLayout,
    write_size: *const WGPUExtent3D,
) {
    let queue = &*queue;
    let destination = &convert_wgpu_image_copy_texture(&*destination);
    let data_slice = unsafe { std::slice::from_raw_parts(data, data_size) };
    let data_layout = &convert_wgpu_texture_data_layout(&*data_layout);
    let write_size = &convert_wgpu_extent_3d(&*write_size);
    match queue.write_texture_with_u8_slice_and_gpu_extent_3d_dict(
        destination,
        data_slice,
        data_layout,
        write_size,
    ) {
        Ok(_) => (),
        Err(e) => panic!("Failed to write texture: {e:?}"),
    }
}

#[no_mangle]
unsafe extern "C" fn wgpuQueueRelease(queue: *mut GpuQueue) {
    drop(Box::from_raw(queue));
}

// Methods of RenderPassEncoder

#[no_mangle]
unsafe extern "C" fn wgpuRenderPassEncoderDraw(
    pass: *mut GpuRenderPassEncoder,
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
) {
    let pass = &*pass;
    pass.draw_with_instance_count_and_first_vertex_and_first_instance(
        vertex_count,
        instance_count,
        first_vertex,
        first_instance,
    );
}

#[no_mangle]
unsafe extern "C" fn wgpuRenderPassEncoderDrawIndexed(
    pass: *mut GpuRenderPassEncoder,
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
) {
    let pass = &*pass;
    pass.draw_indexed_with_instance_count_and_first_index_and_base_vertex_and_first_instance(
        index_count,
        instance_count,
        first_index,
        base_vertex,
        first_instance,
    );
}

#[no_mangle]
unsafe extern "C" fn wgpuRenderPassEncoderEnd(pass: *mut GpuRenderPassEncoder) {
    let pass = &*pass;
    pass.end();
}

#[no_mangle]
unsafe extern "C" fn wgpuRenderPassEncoderSetBindGroup(
    pass: *mut GpuRenderPassEncoder,
    index: u32,
    bind_group: Option<NonNull<GpuBindGroup>>,
    dynamic_offsets_count: usize,
    dynamic_offsets: *const u32,
) {
    let pass = &*pass;
    let bind_group = bind_group.map(|b| b.as_ref());
    if !dynamic_offsets.is_null() && dynamic_offsets_count > 0 {
        let offsets = unsafe { std::slice::from_raw_parts(dynamic_offsets, dynamic_offsets_count) };
        let js_offsets = js_sys::Array::new();
        for &offset in offsets {
            js_offsets.push(&JsValue::from(offset));
        }
        pass.set_bind_group_with_u32_sequence(index, bind_group, &js_offsets.into());
    } else {
        pass.set_bind_group(index, bind_group);
    }
}

#[no_mangle]
unsafe extern "C" fn wgpuRenderPassEncoderSetIndexBuffer(
    pass: *mut GpuRenderPassEncoder,
    buffer: *mut GpuBuffer,
    format: WGPUIndexFormat,
    offset: u64,
    size: u64,
) {
    let pass = &*pass;
    let buffer = &*buffer;
    let format = enum_wgpu_index_format(format);
    pass.set_index_buffer_with_f64_and_f64(buffer, format, offset as f64, size as f64);
}

#[no_mangle]
unsafe extern "C" fn wgpuRenderPassEncoderSetPipeline(
    pass: *mut GpuRenderPassEncoder,
    pipeline: *mut GpuRenderPipeline,
) {
    let pass = &*pass;
    let pipeline = &*pipeline;
    pass.set_pipeline(pipeline);
}

#[no_mangle]
unsafe extern "C" fn wgpuRenderPassEncoderSetScissorRect(
    pass: *mut GpuRenderPassEncoder,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    let pass = &*pass;
    pass.set_scissor_rect(x, y, width, height);
}

#[no_mangle]
unsafe extern "C" fn wgpuRenderPassEncoderSetStencilReference(
    pass: *mut GpuRenderPassEncoder,
    reference: u32,
) {
    let pass = &*pass;
    pass.set_stencil_reference(reference);
}

#[no_mangle]
unsafe extern "C" fn wgpuRenderPassEncoderSetVertexBuffer(
    pass: *mut GpuRenderPassEncoder,
    slot: u32,
    buffer: Option<NonNull<GpuBuffer>>,
    offset: u64,
    size: u64,
) {
    let pass = &*pass;
    let buffer = buffer.map(|b| b.as_ref());
    pass.set_vertex_buffer_with_f64_and_f64(slot, buffer, offset as f64, size as f64);
}

#[no_mangle]
unsafe extern "C" fn wgpuRenderPassEncoderRelease(pass: *mut GpuRenderPassEncoder) {
    drop(Box::from_raw(pass));
}

// Methods of RenderPipeline

#[no_mangle]
unsafe extern "C" fn wgpuRenderPipelineRelease(pipeline: *mut GpuRenderPipeline) {
    drop(Box::from_raw(pipeline));
}

// Methods of Sampler

#[no_mangle]
unsafe extern "C" fn wgpuSamplerRelease(sampler: *mut GpuSampler) {
    drop(Box::from_raw(sampler));
}

// Methods of ShaderModule

#[no_mangle]
unsafe extern "C" fn wgpuShaderModuleRelease(module: *mut GpuShaderModule) {
    drop(Box::from_raw(module));
}

// Methods of Surface

#[no_mangle]
unsafe extern "C" fn wgpuSurfaceConfigure(
    surface: *mut GpuCanvasContext,
    config: *const GpuCanvasConfiguration,
) {
    let surface = &*surface;
    let config = &*config;
    let _ = surface.configure(config);
}

#[no_mangle]
unsafe extern "C" fn wgpuSurfaceGetCurrentTexture(
    surface: *mut GpuCanvasContext,
    surface_texture: *mut WGPUSurfaceTexture,
) {
    let surface = &*surface;
    let surface_texture = &mut *surface_texture;

    match surface.get_current_texture() {
        Ok(texture) => {
            surface_texture.texture = Box::into_raw(Box::new(texture));
            surface_texture.status = 0; // WGPU_SURFACE_TEXTURE_STATUS_SUCCESS
        }
        Err(_) => {
            surface_texture.texture = std::ptr::null_mut();
            surface_texture.status = 3; // WGPU_SURFACE_TEXTURE_STATUS_ERROR
        }
    }
}

#[no_mangle]
unsafe extern "C" fn wgpuSurfaceUnconfigure(surface: *mut GpuCanvasContext) {
    let surface = &*surface;
    surface.unconfigure();
}

// Methods of Texture

#[no_mangle]
unsafe extern "C" fn wgpuTextureCreateView(
    texture: *mut GpuTexture,
    descriptor: *const WGPUTextureViewDescriptor,
) -> *mut GpuTextureView {
    let texture = &*texture;
    let descriptor = convert_wgpu_texture_view_descriptor(&*descriptor);
    descriptor.set_usage(texture.usage());
    let view = texture
        .create_view_with_descriptor(&descriptor)
        .expect("Failed to create texture view");
    Box::into_raw(Box::new(view))
}

#[no_mangle]
unsafe extern "C" fn wgpuTextureDestroy(texture: *mut GpuTexture) {
    let texture = &*texture;
    texture.destroy();
}

#[no_mangle]
unsafe extern "C" fn wgpuTextureGetFormat(texture: *mut GpuTexture) -> WGPUTextureFormat {
    let texture = &*texture;
    enum_wgpu_texture_format_from_gpu(texture.format())
}

#[no_mangle]
unsafe extern "C" fn wgpuTextureGetHeight(texture: *mut GpuTexture) -> u32 {
    let texture = &*texture;
    texture.height()
}

#[no_mangle]
unsafe extern "C" fn wgpuTextureGetWidth(texture: *mut GpuTexture) -> u32 {
    let texture = &*texture;
    texture.width()
}

#[no_mangle]
unsafe extern "C" fn wgpuTextureRelease(texture: *mut GpuTexture) {
    drop(Box::from_raw(texture));
}

// Methods of TextureView

#[no_mangle]
unsafe extern "C" fn wgpuTextureViewRelease(view: *mut GpuTextureView) {
    drop(Box::from_raw(view));
}
