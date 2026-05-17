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
        let obj_dir = &self.workspace.config.layout.objs;

        for file in &self.graph.compile_order {
            let imports = match self.graph.dependecies.get(file) {
                Some(i) => i,
                None => continue,
            };

            let mut stub_paths: Vec<String> = Vec::new();

            for module_name in imports {
                match self.graph.module_map.get(module_name) {
                    Some(dep_file) => {
                        let stub = format!("{}/{}.stub", stubs_dir, module_name);
                        if !std::path::Path::new(&stub).exists() {
                            println!("generating stub for {}", module_name);
                            let status = Command::new("unnc")
                                .arg(dep_file)
                                .arg("-stub")
                                .arg(&stub)
                                .status()
                                .map_err(|e| format!("error: failed to invoke unnc: {}", e))?;
                            if !status.success() {
                                return Err(format!(
                                    "error: stub generation failed for {}",
                                    module_name
                                ));
                            }
                        }
                        stub_paths.push(stub);
                    }
                    None => return Err(format!("error: unknown module '{}'", module_name)),
                }
            }

            // compile this file
            let file_stem = std::path::Path::new(file)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let obj = format!("{}/{}.o", obj_dir, file_stem);

            println!("compiling {} -> {}", file, obj);

            let mut cmd = Command::new("unnc");
            cmd.arg(file).arg("-compile").arg(&obj);

            for stub in &stub_paths {
                cmd.arg("-load").arg(stub);
            }

            let status = cmd
                .status()
                .map_err(|e| format!("error: failed to invoke unnc: {}", e))?;

            if !status.success() {
                return Err(format!("error: compilation failed for {}", file));
            }
        }

        // link if executable
        if self.workspace.config.project.mode == BuildMode::Executable {
            let build_dir = &self.workspace.config.layout.build;
            let name = &self.workspace.config.project.name;
            let output = format!("{}/{}", build_dir, name);

            println!("linking -> {}", output);

            let mut cmd = Command::new("unnc");
            cmd.arg("-link-only").arg("-build").arg(&output);

            // entry obj first
            let entry_stem = std::path::Path::new(&self.workspace.entry)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let entry_obj = format!("{}/{}.o", obj_dir, entry_stem);
            cmd.arg("-link").arg(&entry_obj);

            // rest of objs
            for file in &self.graph.compile_order {
                if file == &self.workspace.entry {
                    continue;
                }
                let file_stem = std::path::Path::new(file)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let obj = format!("{}/{}.o", obj_dir, file_stem);
                cmd.arg("-link").arg(&obj);
            }

            // C libs from belt.lethr [link] section
            if let Some(link) = &self.workspace.config.link {
                for (_name, libs) in &link.links {
                    for lib in libs {
                        cmd.arg("-link").arg(lib);
                    }
                }
            }

            let status = cmd
                .status()
                .map_err(|e| format!("error: failed to invoke unnc's link driver: {}", e))?;

            if !status.success() {
                return Err("error: linking failed".to_string());
            }
        }

        Ok(())
    }
}
