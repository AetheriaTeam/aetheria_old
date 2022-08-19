use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct AccessorRef(pub usize);
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct BufferRef(pub usize);
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct BufferViewRef(pub usize);
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct MeshRef(pub usize);
#[derive(Deserialize, Debug, Clone, Copy)]
pub struct NodeRef(pub usize);

#[derive(Deserialize, Debug)]
pub struct Accessor {
    #[serde(rename = "bufferView")]
    pub buffer_view: BufferViewRef,
    #[serde(rename = "byteOffset")]
    #[serde(default)]
    pub byte_offset: usize,
    #[serde(rename = "componentType")]
    pub component_type: u32,
    pub count: usize,
    #[serde(rename = "type")]
    pub element_type: String,
}

#[derive(Deserialize, Debug)]
pub struct Buffer {
    #[serde(rename = "byteLength")]
    pub byte_length: usize,
}

#[derive(Deserialize, Debug)]
pub struct BufferView {
    pub buffer: BufferRef,
    #[serde(rename = "byteOffset")]
    #[serde(default)]
    pub byte_offset: usize,
    #[serde(rename = "byteLength")]
    pub byte_length: usize,
}

#[derive(Deserialize, Debug)]
pub struct MeshPrimitive {
    pub attributes: HashMap<String, AccessorRef>,
    #[serde(default)]
    pub indices: Option<AccessorRef>,
}

#[derive(Deserialize, Debug)]
pub struct Mesh {
    pub primitives: Vec<MeshPrimitive>,
}

#[derive(Deserialize, Debug)]
pub struct Node {
    #[serde(default)]
    pub children: Vec<NodeRef>,
    #[serde(default)]
    pub matrix: Option<[f32; 16]>,
    #[serde(default)]
    pub translation: Option<[f32; 3]>,
    #[serde(default)]
    pub rotation: Option<[f32; 4]>,
    #[serde(default)]
    pub scale: Option<[f32; 3]>,
    #[serde(default)]
    pub mesh: Option<MeshRef>,
}

#[derive(Deserialize, Debug)]
pub struct Scene {
    pub nodes: Vec<NodeRef>
}

#[derive(Deserialize, Debug)]
pub struct Gltf {
    #[serde(default)]
    pub accessors: Vec<Accessor>,
    #[serde(default)]
    pub buffers: Vec<Buffer>,
    #[serde(rename = "bufferViews")]
    #[serde(default)]
    pub buffer_views: Vec<BufferView>,
    #[serde(default)]
    pub meshes: Vec<Mesh>,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub scenes: Vec<Scene>,
    #[serde(default)]
    pub scene: usize
}
