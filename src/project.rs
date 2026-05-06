use crate::config::{self, BuildMode, Config};
use std::{fs, path::Path};

pub struct Workspace {
    pub source_files: Vec<String>,
    pub entry: String,
    pub config: Config,
}

//Helpers
fn scan_directory(dir: &str, unn_files: &mut Vec<String>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            scan_directory(path.to_str().unwrap(), unn_files)?;
        } else if path.extension() == Some("unn".as_ref()) {
            unn_files.push(path.to_string_lossy().into_owned());
        }
    }

    Ok(())
}

fn get_source_files(config: &Config) -> Result<Vec<String>, std::io::Error> {
    let mut unn_files = Vec::new();
    scan_directory(&config.layout.src, &mut unn_files)?;
    Ok(unn_files)
}

impl Workspace {
    pub fn build(config: &Config) -> Result<Workspace, String> {
        let mut workspace = Workspace {
            source_files: Vec::new(),
            entry: String::new(),
            config: config.clone(),
        };

        workspace.source_files = match get_source_files(config) {
            Ok(res) => res,
            Err(e) => return Err(format!("error: failed to get source files {}", e)),
        };

        if config.project.mode == BuildMode::Executable {
            workspace.entry = match &config.project.entry {
                Some(entry) => {
                    if !Path::new(entry).exists() {
                        return Err(format!("error: failed to get entry file {}", entry));
                    } else {
                        entry.clone()
                    }
                }
                None => {
                    return Err("error: mode is executable but entry point doesnt exist".to_string());
                }
            }
        }

        Ok(workspace)
    }
}
