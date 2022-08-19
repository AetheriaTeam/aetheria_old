mod json;

use eyre::ensure;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io::{Cursor, Read};
use std::path::Path;
use std::str;
use vulkano::buffer::BufferContents;

use crate::types::mesh::MeshData;
use crate::vulkan::vertex::Vertex;

const GLB_MAGIC: u32 = 0x46_54_6C_67;

#[derive(Clone, Copy, PartialEq, FromPrimitive)]
enum GlbChunkType {
    Json = 0x4E_4F_53_4A,
    Binary = 0x00_4E_49_42,
}

struct GlbChunk {
    chunk_type: GlbChunkType,
    data: Vec<u8>,
}

struct GlbChunks(Vec<GlbChunk>);
impl GlbChunks {
    fn get_chunk(&self, chunk_type: GlbChunkType) -> Option<Vec<u8>> {
        self.0
            .iter()
            .find(|chunk| chunk.chunk_type == chunk_type)
            .map(|chunk| chunk.data.clone())
    }
}

#[derive(Debug)]
pub struct Gltf {
    gltf: json::Gltf,
    buffer: Vec<u8>,
}

impl Gltf {
    #[doc = "# Errors"]
    #[doc = "Errors if the GLB failed to decode, or if it invalid based on the specification"]
    pub fn load(path: &Path) -> eyre::Result<Self> {
        let mut buffer = Cursor::new(std::fs::read(path)?);

        let magic = buffer.read_u32()?;
        let version = buffer.read_u32()?;
        let _length = buffer.read_u32()?;

        ensure!(
            magic == GLB_MAGIC,
            "{} is not a valid GLB file due to the missing magic number",
            path.display()
        );
        ensure!(
            version == 2,
            "Aether only support GLB version 2, you are trying to use version {}",
            version
        );

        let mut chunks = GlbChunks(Vec::new());

        while let Ok(chunk_length) = buffer.read_u32() {
            let chunk_type = buffer.read_u32()?;
            let mut chunk_data = vec![0; chunk_length as usize];
            buffer.read_exact(&mut chunk_data)?;

            chunks.0.push(GlbChunk {
                chunk_type: match FromPrimitive::from_u32(chunk_type) {
                    Some(value) => value,
                    None => {
                        return Err(eyre::eyre!("{} is not a valid GLB chunk type", chunk_type))
                    }
                },
                data: chunk_data,
            });
        }

        let gltf = Self::load_json_chunk(
            &chunks
                .get_chunk(GlbChunkType::Json)
                .ok_or_else(|| eyre::eyre!("GLB file {} has no JSON chunk", path.display()))?,
        )?;
        let binary_data = chunks.get_chunk(GlbChunkType::Binary).ok_or_else(|| eyre::eyre!(
            "GLB file {} has no binary chunk",
            path.display()
        ))?;

        Ok(Self {
            gltf,
            buffer: binary_data,
        })
    }

    #[must_use]
    pub fn load_accessor_data(&self, accessor_ref: json::AccessorRef) -> &[u8] {
        let accessor = &self.gltf.accessors[accessor_ref.0];
        let buffer_view = &self.gltf.buffer_views[accessor.buffer_view.0];

        let offset = accessor.byte_offset + buffer_view.byte_offset;
        let length = Self::get_accessor_length(accessor);
        &self.buffer[offset..(offset + length)]
    }

    #[must_use]
    #[doc = "# Panics"]
    #[doc = "Panics if converting the binary data to vertices and indices fails, or if either are missing from a mesh"]
    pub fn to_meshes(&self) -> Vec<MeshData> {
        self.gltf
            .meshes
            .iter()
            .flat_map(|mesh| {
                mesh.primitives
                    .iter()
                    .map(|primitive| {
                        let vertices = self.load_accessor_data(primitive.attributes["POSITION"]);
                        let vertices = vertices
                            .chunks(12)
                            .map(|bytes| match Vertex::from_bytes(bytes) {
                                Ok(vertex) => *vertex,
                                Err(e) => panic!("Failed to turn {:?} into a Vertex due to {:?}", bytes, e)
                            })
                            .collect();

                        let indices = self.load_accessor_data(match primitive.indices {
                            Some(indices) => indices,
                            None => panic!("Aether currently doesn't support glTF meshes without index buffers")
                        });
                        let indices = indices
                            .chunks(2)
                            .map(|bytes| u32::from(match u16::from_bytes(bytes) {
                                Ok(index) => *index,
                                Err(e) => panic!("Failed to turn {:?} into a index due to {:?}", bytes, e)
                            }))
                            .collect();
                        MeshData { vertices, indices }
                    })
                    .collect::<Vec<MeshData>>()
            })
            .collect()
    }

    fn load_json_chunk(data: &[u8]) -> eyre::Result<json::Gltf> {
        Ok(serde_json::from_str(str::from_utf8(data)?)?)
    }

    fn get_accessor_length(accessor: &json::Accessor) -> usize {
        let element_type = match accessor.element_type.as_str() {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" | "MAT2" => 4,
            "MAT3" => 9,
            "MAT4" => 16,
            _ => panic!(
                "Invalid glTF accessor element type {}",
                accessor.element_type
            ),
        };

        let component_type = match accessor.component_type {
            5120 | 5121 => 1,
            5122 | 5123 => 2,
            5125 | 5126 => 4,
            _ => panic!(
                "Invalid glTF accessor component type {}",
                accessor.component_type
            ),
        };

        element_type * component_type * accessor.count
    }
}

trait GlbReader {
    fn read_u32(&mut self) -> eyre::Result<u32>;
}

impl GlbReader for Cursor<Vec<u8>> {
    fn read_u32(&mut self) -> eyre::Result<u32> {
        let mut bytes = [0; 4];
        self.read_exact(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }
}
