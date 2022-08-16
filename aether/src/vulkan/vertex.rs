#![allow(clippy::use_self)]

use bytemuck::{Pod, Zeroable};
use serde::{Serialize, Deserialize};

#[repr(C)]
#[derive(Debug, Default, Clone, Pod, Zeroable, Copy, Deserialize, Serialize)]
pub struct Vertex {
    #[serde(rename = "p")]
    pub position: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position);
