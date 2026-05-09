use std::process::Command;

use crate::config::BuildMode;
use crate::{graph::Graph, project::Workspace};

pub struct Builder {
    pub workspace: Workspace,
    pub graph: Graph,
}

impl Builder {
    pub fn new(workspace: Workspace) -> Result<Self, String> {
        let builder = Builder {
            graph: match Graph::build(&workspace.source_files) {
                Ok(g) => g,
                Err(_) => return Err("error: failed to get source files".to_string()),
            },
            workspace: workspace,
        };

        Ok(builder)
    }

    pub fn build(&self) -> Result<(), String> {
        let stubs_dir = &self.workspace.config.layout.stubs;

        for file in &self.graph.compile_order {
            // get imports for this file
            let imports = match self.graph.dependecies.get(file) {
                Some(i) => i,
                None => continue,
            };

            // resolve each import to a stub
            let mut stub_paths: Vec<String> = Vec::new();
            for module_name in imports {
                match self.graph.module_map.get(module_name) {
                    Some(_file_path) => {
                        let stub = format!("{}/{}.stub", stubs_dir, module_name);
                        // if stub doesn't exist generate it
                        if !std::path::Path::new(&stub).exists() {
                            println!("TODO: generate stub for {}", module_name);
                            // unnc --file file_path --stub-out stubs/
                        }
                        stub_paths.push(stub);
                    }
                    None => return Err(format!("error: unknown module '{}'", module_name)),
                }
            }

            // compile this file with its stubs
            println!("TODO: compile {} with stubs {:?}", file, stub_paths);
            // unnc --file file --load stub1 --load stub2
        }

        // link if executable
        if self.workspace.config.project.mode == BuildMode::Executable {
            let entry = &self.workspace.entry;
            let build_dir = &self.workspace.config.layout.build;
            let name = &self.workspace.config.project.name;
            let output = format!("{}/{}", build_dir, name);

            println!("TODO: link {} -> {}", entry, output);
            // unnc --link --entry entry --obj-dir obj/ --out output
        }

        Ok(())
    }
}
