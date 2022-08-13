use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use ciborium::{de::from_reader, ser::into_writer};
use eyre::Context;
use serde::{Deserialize, Serialize};

use crate::vulkan::vertex::Vertex;

#[derive(Serialize, Deserialize)]
pub struct AMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl AMesh {
    #[doc = "# Errors"]
    #[doc = "Errors if saving the mesh fails"]
    pub fn new(path: &Path, vertices: Vec<Vertex>, indices: Vec<u32>) -> eyre::Result<Self> {
        let mesh = Self { vertices, indices };
        mesh.save(path)?;
        Ok(mesh)
    }

    #[doc = "# Errors"]
    #[doc = "Errors if loading the mesh fails"]
    pub fn load(path: &Path) -> eyre::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        from_reader(reader).wrap_err("Failed to load .amesh file")
    }

    #[doc = "# Errors"]
    #[doc = "Errors if saving the mesh fails"]
    pub fn save(&self, path: &Path) -> eyre::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        into_writer(self, writer).wrap_err("Failed to save .amesh file")
    }
}
