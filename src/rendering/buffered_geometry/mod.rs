use bytemuck::Pod;
use serde::Serialize;
use serde_repr::Serialize_repr;

/// Sync with WebGL type values.
/// For possible values see: https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/vertexAttribPointer
/// For numeric values see: https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/Constants
#[repr(u32)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize_repr)]
pub enum AttributeType {
    Byte = 0x1400,
    UnsignedByte = 0x1401,
    Short = 0x1402,
    UnsignedShort = 0x1403,
    Float = 0x1406,
    HalfFloat = 0x140B,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum VertexFormat {
    /// Two unsigned bytes (u8). `uvec2` in shaders.
    Uint8x2 = 0,
    /// Four unsigned bytes (u8). `uvec4` in shaders.
    Uint8x4 = 1,
    /// Two signed bytes (i8). `ivec2` in shaders.
    Sint8x2 = 2,
    /// Four signed bytes (i8). `ivec4` in shaders.
    Sint8x4 = 3,
    /// Two unsigned bytes (u8). [0, 255] converted to float [0, 1] `vec2` in shaders.
    Unorm8x2 = 4,
    /// Four unsigned bytes (u8). [0, 255] converted to float [0, 1] `vec4` in shaders.
    Unorm8x4 = 5,
    /// Two signed bytes (i8). [-127, 127] converted to float [-1, 1] `vec2` in shaders.
    Snorm8x2 = 6,
    /// Four signed bytes (i8). [-127, 127] converted to float [-1, 1] `vec4` in shaders.
    Snorm8x4 = 7,
    /// Two unsigned shorts (u16). `uvec2` in shaders.
    Uint16x2 = 8,
    /// Four unsigned shorts (u16). `uvec4` in shaders.
    Uint16x4 = 9,
    /// Two signed shorts (i16). `ivec2` in shaders.
    Sint16x2 = 10,
    /// Four signed shorts (i16). `ivec4` in shaders.
    Sint16x4 = 11,
    /// Two unsigned shorts (u16). [0, 65535] converted to float [0, 1] `vec2` in shaders.
    Unorm16x2 = 12,
    /// Four unsigned shorts (u16). [0, 65535] converted to float [0, 1] `vec4` in shaders.
    Unorm16x4 = 13,
    /// Two signed shorts (i16). [-32767, 32767] converted to float [-1, 1] `vec2` in shaders.
    Snorm16x2 = 14,
    /// Four signed shorts (i16). [-32767, 32767] converted to float [-1, 1] `vec4` in shaders.
    Snorm16x4 = 15,
    /// Two half-precision floats (no Rust equiv). `vec2` in shaders.
    Float16x2 = 16,
    /// Four half-precision floats (no Rust equiv). `vec4` in shaders.
    Float16x4 = 17,
    /// One single-precision float (f32). `float` in shaders.
    Float32 = 18,
    /// Two single-precision floats (f32). `vec2` in shaders.
    Float32x2 = 19,
    /// Three single-precision floats (f32). `vec3` in shaders.
    Float32x3 = 20,
    /// Four single-precision floats (f32). `vec4` in shaders.
    Float32x4 = 21,
}

impl VertexFormat {
    /// Returns the byte size of the format.
    pub const fn byte_size(&self) -> u32 {
        match self {
            Self::Uint8x2 | Self::Sint8x2 | Self::Unorm8x2 | Self::Snorm8x2 => 2,
            Self::Uint8x4
            | Self::Sint8x4
            | Self::Unorm8x4
            | Self::Snorm8x4
            | Self::Uint16x2
            | Self::Sint16x2
            | Self::Unorm16x2
            | Self::Snorm16x2
            | Self::Float16x2
            | Self::Float32 => 4,
            Self::Uint16x4
            | Self::Sint16x4
            | Self::Unorm16x4
            | Self::Snorm16x4
            | Self::Float16x4
            | Self::Float32x2 => 8,
            Self::Float32x3 => 12,
            Self::Float32x4 => 16,
        }
    }

    pub const fn normalized(&self) -> bool {
        match self {
            Self::Unorm16x2
            | Self::Unorm16x4
            | Self::Unorm8x2
            | Self::Unorm8x4
            | Self::Snorm16x2
            | Self::Snorm16x4
            | Self::Snorm8x2
            | Self::Snorm8x4 => true,
            _ => false,
        }
    }

    pub const fn count(&self) -> u32 {
        match self {
            Self::Float32 => 1,
            Self::Uint8x2
            | Self::Sint8x2
            | Self::Unorm8x2
            | Self::Snorm8x2
            | Self::Uint16x2
            | Self::Sint16x2
            | Self::Unorm16x2
            | Self::Snorm16x2
            | Self::Float16x2
            | Self::Float32x2 => 2,
            Self::Float32x3 => 3,
            Self::Uint8x4
            | Self::Sint8x4
            | Self::Unorm8x4
            | Self::Snorm8x4
            | Self::Uint16x4
            | Self::Sint16x4
            | Self::Unorm16x4
            | Self::Snorm16x4
            | Self::Float16x4
            | Self::Float32x4 => 4,
        }
    }

    pub const fn attr_type(&self) -> AttributeType {
        match self {
            Self::Uint8x2 | Self::Unorm8x2 | Self::Uint8x4 | Self::Unorm8x4 => {
                AttributeType::UnsignedByte
            }
            Self::Sint8x2 | Self::Snorm8x2 | Self::Sint8x4 | Self::Snorm8x4 => AttributeType::Byte,
            Self::Uint16x2 | Self::Uint16x4 | Self::Unorm16x2 | Self::Unorm16x4 => {
                AttributeType::UnsignedShort
            }
            Self::Sint16x2 | Self::Sint16x4 | Self::Snorm16x2 | Self::Snorm16x4 => {
                AttributeType::Short
            }
            Self::Float16x2 | Self::Float16x4 => AttributeType::HalfFloat,
            Self::Float32x2 | Self::Float32x3 | Self::Float32x4 | Self::Float32 => {
                AttributeType::Float
            }
        }
    }
}

pub struct VertexType {
    pub name: &'static str,
    pub format: VertexFormat,
}

impl VertexType {
    pub fn new(name: &'static str, format: VertexFormat) -> VertexType {
        VertexType { name, format }
    }
}

pub fn vert_type(name: &'static str, format: VertexFormat) -> VertexType {
    VertexType::new(name, format)
}

pub trait WithVertexLayout {
    fn vertex_layout() -> Vec<VertexType>;
}

#[derive(Clone, Serialize, Debug)]
pub struct AttributeLayout {
    pub name: &'static str,
    pub size: u32,
    pub attr_type: AttributeType,
    pub normalized: bool,
    pub offset: u32,
}

#[derive(Clone, Serialize, Debug)]
pub struct BufferedGeometry {
    pub buffer: Vec<u8>,
    pub indices: Option<Vec<u32>>,
    pub vertex_size: u32,
    pub vertex_layout: Vec<AttributeLayout>,
}

pub struct BufferedGeometryLayout {
    pub vertex_size: u32,
    pub vertex_layout: Vec<AttributeLayout>,
}

pub fn create_buffered_geometry_layout(layout: Vec<VertexType>) -> BufferedGeometryLayout {
    let mut vertex_layout = vec![];
    let mut vertex_size = 0;

    for vert_type in layout {
        let format = vert_type.format;
        vertex_layout.push(AttributeLayout {
            name: vert_type.name,
            size: format.count(),
            attr_type: format.attr_type(),
            normalized: format.normalized(),
            offset: vertex_size,
        });
        vertex_size += format.byte_size();
    }

    BufferedGeometryLayout {
        vertex_layout,
        vertex_size,
    }
}

pub trait ToBufferedGeometry {
    fn to_buffered_geometry(&self, layout: Vec<VertexType>) -> BufferedGeometry;
}

pub trait ToBufferedVertexData<T: Pod> {
    fn to_buffered_vertex_data(&self) -> T;
}

pub trait BufferedVertexData: Pod + Clone + Copy {}

impl<T: BufferedVertexData> ToBufferedVertexData<T> for T {
    fn to_buffered_vertex_data(&self) -> T {
        self.clone()
    }
}
