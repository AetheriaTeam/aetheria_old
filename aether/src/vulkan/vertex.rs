#![allow(clippy::use_self)]

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Default, Clone, Pod, Zeroable, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);
