use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use ciborium::{de::from_reader, ser::into_writer};
use eyre::Context;

use crate::types::mesh::MeshData;

pub trait AMesh : Sized {
    fn load(path: &Path) -> eyre::Result<Self>;
    fn save(&self, path: &Path) -> eyre::Result<()>;
}

impl AMesh for MeshData {
    #[doc = "# Errors"]
    #[doc = "Errors if loading the mesh fails"]
    fn load(path: &Path) -> eyre::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        from_reader(reader).wrap_err("Failed to load .amesh file")
    }

    #[doc = "# Errors"]
    #[doc = "Errors if saving the mesh fails"]
    fn save(&self, path: &Path) -> eyre::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        into_writer(self, writer).wrap_err("Failed to save .amesh file")
    }
}
