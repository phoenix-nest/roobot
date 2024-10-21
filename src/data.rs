use std::{
    fs::{self, read_to_string, File},
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Context, Result};

const STATE_FILE_NAME: &str = "state.json";

pub(crate) struct Datastore {
    base_path: PathBuf,
}
impl Datastore {
    pub(crate) fn new(path: &Path) -> Datastore {
        Datastore {
            base_path: PathBuf::from(path),
        }
    }

    pub(crate) fn load_state_file(&self) -> Result<String> {
        let statefile = self.base_path.join(STATE_FILE_NAME);
        if !statefile.exists() {
            File::create_new(&statefile)?;
        }
        read_to_string(&statefile)
            .wrap_err(format!("Reading state file at {}", statefile.display()))
    }

    pub(crate) fn save_state_file(&self, data: &str) -> Result<()> {
        let statefile = self.base_path.join(STATE_FILE_NAME);
        fs::write(&statefile, data)
            .wrap_err(format!("Writing state tile to {}", statefile.display()))
    }
}
