use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
};

use eyre::{ensure, eyre, Result};
use nalgebra_glm::Mat4;

const GLTF_MAGIC: u32 = 0x46_54_6C_67;
const GLTF_JSON_CHUNK_TYPE: u32 = 0x4E_4F_53_4A;
const GLTF_BIN_CHUNK_TYPE: u32 = 0x00_4E_49_42;

mod json {
    use nalgebra_glm::{
        identity, make_mat4, make_quat, make_vec3, matrix_comp_mult, quat_to_mat4, scale,
        translate, Mat4,
    };
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Clone)]
    pub struct Accessor {
        #[serde(rename = "bufferView")]
        pub buffer_view: u32,
        #[serde(rename = "byteOffset")]
        #[serde(default)]
        pub byte_offfset: u32,
        #[serde(rename = "componentType")]
        pub component_type: u32,
        #[serde(default)]
        pub normalized: bool,
        pub count: u32,
        #[serde(rename = "type")]
        pub element_type: String,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Buffer {
        #[serde(rename = "byteLength")]
        pub byte_length: u32,
        pub uri: Option<String>,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct BufferView {
        pub buffer: u32,
        #[serde(rename = "byteOffset")]
        #[serde(default)]
        pub byte_offset: u32,
        #[serde(rename = "byteLength")]
        pub byte_length: u32,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct MeshAttributes {
        #[serde(rename = "POSITION")]
        pub position: u32,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct MeshPrimitive {
        pub attributes: MeshAttributes,
        pub indices: u32,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Mesh {
        pub primitives: Vec<MeshPrimitive>,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Node {
        #[serde(default)]
        pub children: Vec<u32>,
        matrix: Option<[f32; 16]>,
        translation: Option<[f32; 3]>,
        rotation: Option<[f32; 4]>,
        scale: Option<[f32; 3]>,
        pub mesh: Option<u32>,
    }

    impl Node {
        pub fn get_matrix(&self) -> Mat4 {
            match self.matrix {
                Some(values) => make_mat4(&values),
                None => {
                    let mut matrix: Mat4 = identity();

                    if let Some(values) = self.scale {
                        matrix = scale(&matrix, &make_vec3(&values));
                    }

                    if let Some(values) = self.rotation {
                        matrix = matrix_comp_mult(&quat_to_mat4(&make_quat(&values)), &matrix);
                    }

                    if let Some(values) = self.translation {
                        matrix = translate(&matrix, &make_vec3(&values));
                    }

                    matrix
                }
            }
        }
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Scene {
        pub nodes: Vec<u32>,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Gltf {
        pub accessors: Vec<Accessor>,
        pub buffers: Vec<Buffer>,
        #[serde(rename = "bufferViews")]
        pub buffer_views: Vec<BufferView>,
        pub meshes: Vec<Mesh>,
        pub nodes: Vec<Node>,
        pub scenes: Vec<Scene>,
    }
}

pub struct AccessorRef(usize);

impl AccessorRef {
    #[must_use]
    pub fn get_data<'a>(&self, gltf: &'a Gltf) -> &'a [u8] {
        let accessor = &gltf.accessors[self.0];
        let buffer_view = &gltf.buffer_views[accessor.buffer_view.0];
        let offset = accessor.offset + buffer_view.offset;
        let length = accessor.component_type.get_size() * accessor.element_type.get_size() * accessor.count;
        println!("Component type: {}, Element type: {}, Count: {}", accessor.component_type.get_size(), accessor.element_type.get_size(), accessor.count);
        println!("Offset: {}, Length: {}", offset, length);
        &gltf.buffer.data[offset..(offset + length)]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AccessorElementType {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

impl AccessorElementType {
    const fn get_size(self) -> usize {
        match self {
            Self::Scalar => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
            Self::Mat2 => 4,
            Self::Mat3 => 9,
            Self::Mat4 => 16
        }
    }
}

impl Into<AccessorElementType> for String {
    fn into(self) -> AccessorElementType {
        match self.as_str() {
            "SCALAR" => AccessorElementType::Scalar,
            "VEC2" => AccessorElementType::Vec2,
            "VEC3" => AccessorElementType::Vec3,
            "VEC4" => AccessorElementType::Vec4,
            "MAT2" => AccessorElementType::Mat2,
            "MAT3" => AccessorElementType::Mat3,
            "MAT4" => AccessorElementType::Mat4,
            _ => panic!("Invalid AcccessorElementType {}", self),
        }
    }
}

#[derive(Clone, Copy)]
pub enum AccessorComponentType {
    I8,
    U8,
    I16,
    U16,
    U32,
    F32
}

impl AccessorComponentType {
    pub fn get_size(&self) -> usize {
        match self {
            Self::I8 => 1,
            Self::U8 => 1,
            Self::I16 => 2,
            Self::U16 => 2,
            Self::U32 => 4,
            Self::F32 => 4
        }
    }
}

impl Into<AccessorComponentType> for u32 {
fn into(self) -> AccessorComponentType {
    match self {
        5120 => AccessorComponentType::I8,
        5121 => AccessorComponentType::U8,
        5122 => AccessorComponentType::I16,
        5123 => AccessorComponentType::U16,
        5125 => AccessorComponentType::U32,
        5126 => AccessorComponentType::F32,
        _ => panic!("Invalid AccessorComponentType {}", self)
    }
}
}

pub struct Accessor {
    buffer_view: BufferViewRef,
    offset: usize,
    component_type: AccessorComponentType,
    normalized: bool,
    count: usize,
    element_type: AccessorElementType,
}

pub struct BufferRef(usize);

pub struct Buffer {
    length: usize,
    data: Vec<u8>,
}

pub struct BufferViewRef(usize);

pub struct BufferView {
    buffer: BufferRef,
    offset: usize,
    length: usize,
}

pub struct MeshRef(usize);
impl MeshRef {
    #[must_use]
    pub fn get<'a>(&self, gltf: &'a Gltf) -> &'a Mesh {
        &gltf.meshes[self.0]
    }
}

pub struct MeshAttributes {
    pub position: AccessorRef,
}

pub struct MeshPrimitive {
    pub attributes: MeshAttributes,
    pub indices: AccessorRef
}

pub struct Mesh {
    pub primitives: Vec<MeshPrimitive>,
}

pub struct NodeRef(usize);
impl NodeRef {
    #[must_use]
    pub fn get<'a>(&self, gltf: &'a Gltf) -> &'a Node {
        &gltf.nodes[self.0]
    }
}

pub struct Node {
    children: Vec<NodeRef>,
    matrix: Mat4,
    mesh: Option<MeshRef>,
}

pub struct Scene {
    nodes: Vec<NodeRef>,
}

pub struct Gltf {
    accessors: Vec<Accessor>,
    buffer: Buffer,
    buffer_views: Vec<BufferView>,
    pub meshes: Vec<Mesh>,
    nodes: Vec<Node>,
    scenes: Vec<Scene>,
}

impl Gltf {
    pub fn load(path: &Path) -> Result<Gltf> {
        let mut file: Cursor<Vec<u8>> = Cursor::new(fs::read(path)?);

        let magic = file.read_u32();
        let version = file.read_u32();
        let _length = file.read_u32();

        ensure!(
            magic == GLTF_MAGIC,
            "GLB magic number is incorrect, file is likely not a GLB file"
        );
        ensure!(
            version == 2,
            "Aether only supports glTF version 2, you are importing a file of glTF version {}",
            version
        );

        let json_chunk_length = file.read_u32();
        let json_chunk_type = file.read_u32();

        ensure!(
            json_chunk_type == GLTF_JSON_CHUNK_TYPE,
            "GLB file is invalid, first chunk type should be {} however it is {}",
            GLTF_JSON_CHUNK_TYPE,
            json_chunk_type
        );

        let mut json_chunk_data: Vec<u8> = vec![0; json_chunk_length as usize];
        file.read_exact(&mut json_chunk_data)?;
        let data: json::Gltf = serde_json::from_slice(&json_chunk_data)?;

        let binary_chunk_length = file.read_u32();
        let binary_chunk_type = file.read_u32();

        ensure!(
            binary_chunk_type == GLTF_BIN_CHUNK_TYPE,
            "GLB file is invalid, second chunk type should be {} however it is {}",
            GLTF_BIN_CHUNK_TYPE,
            binary_chunk_type
        );

        let mut binary_chunk_data = Vec::with_capacity(binary_chunk_length as usize);
        let binary_chunk_data_size = file.read_to_end(&mut binary_chunk_data)?;
        println!("Expected: {}, Got: {}", binary_chunk_length, binary_chunk_data_size);

        let mut gltf = Self {
            accessors: Vec::new(),
            buffer: Gltf::load_buffers(binary_chunk_data, &data).unwrap(),
            buffer_views: Vec::new(),
            meshes: Vec::new(),
            nodes: Vec::new(),
            scenes: Vec::new(),
        };

        gltf.load_buffer_views(&data);
        gltf.load_accessors(&data);
        gltf.load_meshes(&data);
        gltf.load_nodes(&data);
        gltf.load_scenes(&data);

        Ok(gltf)
    }

    fn load_buffers(binary_chunk_data: Vec<u8>, data: &json::Gltf) -> Result<Buffer> {
        // TODO: Support loading .bin files instead of only the binary chunk of the .glb file
        ensure!(
            data.buffers.len() <= 1,
            "Aether currently only support loading data from the .glb binary chunk"
        );

        Ok(Buffer {
            length: data.buffers[0].byte_length as usize,
            data: binary_chunk_data,
        })
    }

    fn load_buffer_views(&mut self, data: &json::Gltf) {
        self.buffer_views.reserve_exact(data.buffer_views.len());

        for buffer_view in &data.buffer_views {
            self.buffer_views.push(BufferView {
                buffer: BufferRef(buffer_view.buffer as usize),
                length: buffer_view.byte_length as usize,
                offset: buffer_view.byte_offset as usize,
            });
        }
    }

    fn load_accessors(&mut self, data: &json::Gltf) {
        self.accessors.reserve_exact(data.accessors.len());

        for accessor in &data.accessors {
            self.accessors.push(Accessor {
                buffer_view: BufferViewRef(accessor.buffer_view as usize),
                offset: accessor.byte_offfset as usize,
                component_type: accessor.component_type.into(),
                normalized: accessor.normalized,
                count: accessor.count as usize,
                element_type: accessor.element_type.clone().into(),
            });
        }
    }

    fn load_meshes(&mut self, data: &json::Gltf) {
        self.meshes.reserve_exact(data.meshes.len());

        for mesh in &data.meshes {
            self.meshes.push(Mesh {
                primitives: mesh
                    .primitives
                    .iter()
                    .map(|primitive| MeshPrimitive {
                        attributes: MeshAttributes {
                            position: AccessorRef(primitive.attributes.position as usize),
                        },
                        indices: AccessorRef(primitive.indices as usize)
                    })
                    .collect(),
            });
        }
    }

    fn load_nodes(&mut self, data: &json::Gltf) {
        self.nodes.reserve_exact(data.nodes.len());

        for node in &data.nodes {
            self.nodes.push(Node {
                children: node
                    .children
                    .iter()
                    .map(|child| NodeRef(*child as usize))
                    .collect(),
                matrix: node.get_matrix(),
                mesh: node.mesh.map(|mesh| MeshRef(mesh as usize))
            });
        }
    }

    fn load_scenes(&mut self, data: &json::Gltf) {
        self.scenes.reserve_exact(data.scenes.len());

        for scene in &data.scenes {
            self.scenes.push(Scene {
                nodes: scene
                    .nodes
                    .iter()
                    .map(|node| NodeRef(*node as usize))
                    .collect(),
            });
        }
    }
}

trait ByteReader {
    fn read_u32(&mut self) -> u32;
}

impl ByteReader for Cursor<Vec<u8>> {
    fn read_u32(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.read_exact(&mut bytes);
        u32::from_le_bytes(bytes)
    }
}
