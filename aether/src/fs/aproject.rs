use std::{fs::File, path::Path, io::{BufReader, BufWriter, Write}};

use ciborium::{de::from_reader, ser::into_writer};
use eyre::Context;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AProject {
    name: String,
}

impl AProject {
    #[doc = "# Errors"]
    #[doc = "Errors if saving the project fails"]
    pub fn new(path: &Path, name: String) -> eyre::Result<Self> {
        let project = Self { name };
        project.save(path)?;
        Ok(project)
    }

    #[doc = "# Errors"]
    #[doc = "Errors if reading the .aproject file fails"]
    pub fn load(path: &Path) -> eyre::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        from_reader(reader).wrap_err("Failed to read .aproject file")
    }

    #[doc = "# Errors"]
    #[doc = "Errors if saving to the .aproject fails"]
    pub fn save(&self, path: &Path) -> eyre::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        into_writer(self, writer.get_ref()).wrap_err("Failed to save .aproject file")?;
        writer.flush()?;
        Ok(())
    }
}
