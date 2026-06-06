use std::process::Command;

use crate::config::{BuildMode, FreeStanding};
use crate::lock::LockFile;
use crate::resolver::Resolver;
use crate::{graph::Graph, project::Workspace};

fn obj_extension() -> &'static str {
    match std::env::consts::OS {
        "windows" => ".obj",
        _ => ".o",
    }
}

fn exe_extension() -> &'static str {
    std::env::consts::EXE_SUFFIX
}

fn target_obj_extension(target: &str) -> &str {
    if target.contains("windows") {
        ".obj"
    } else {
        ".o"
    }
}

fn target_exe_extension(target: &str) -> &str {
    if target.contains("windows") {
        ".exe"
    } else if target.contains("none") {
        ".elf"
    } else {
        ""
    }
}

pub struct Builder {
    pub workspace: Workspace,
    pub graph: Graph,
}

impl Builder {
    pub fn new(workspace: Workspace) -> Result<Self, String> {
        let builder = Builder {
            graph: match Graph::build(&workspace.source_files) {
                Ok(g) => g,
                Err(e) => return Err(format!("error: failed to get source files: {}", e)),
            },
            workspace: workspace,
        };

        Ok(builder)
    }

    pub fn build(&self) -> Result<(), String> {
        let stubs_dir = &self.workspace.config.layout.stubs;
        let obj_dir = &self.workspace.config.layout.objs;
        let target = &self.workspace.config.project.target;

        let obj_ext = match target {
            Some(t) => target_obj_extension(t),
            None => obj_extension(),
        };
        let exe_ext = match target {
            Some(t) => target_exe_extension(t),
            None => exe_extension(),
        };

        let mut lock = LockFile::load("belt.lock");
        let mut dirty_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut resolver = Resolver::new();
        let mut dependency_objects: Vec<String> = Vec::new();

        // pre-resolve all external modules before compilation starts
        for module_name in &self.graph.unresolved {
            let found = resolver.resolve_project(module_name, &self.workspace.config)?;
            if !found {
                return Err(format!(
                    "error: unknown module '{}' — not found in local workspace, std, or dependencies",
                    module_name
                ));
            }
            if let Some(dep) = resolver.resolved.get(module_name) {
                dependency_objects.extend(dep.artifacts.clone());
            }
        }

        // compile loop
        for file in &self.graph.compile_order {
            let imports = match self.graph.dependecies.get(file) {
                Some(i) => i,
                None => continue,
            };

            let module_name = self
                .graph
                .module_map
                .iter()
                .find(|(_, v)| *v == file)
                .map(|(k, _)| k.clone())
                .unwrap_or_else(|| {
                    std::path::Path::new(file)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });

            let is_dirty = lock.is_dirty(&module_name, file, imports, &dirty_set);

            let mut stub_paths: Vec<String> = Vec::new();

            for module_name_dep in imports {
                match self.graph.module_map.get(module_name_dep) {
                    Some(dep_file) => {
                        // local module
                        let stub = format!("{}/{}.stub", stubs_dir, module_name_dep);
                        if !std::path::Path::new(&stub).exists()
                            || dirty_set.contains(module_name_dep)
                        {
                            println!("generating stub for {}", module_name_dep);
                            let mut cmd = Command::new("unnc");
                            cmd.arg(dep_file).arg("-stub").arg(&stub);
                            if let Some(t) = target {
                                cmd.arg("-target").arg(t);
                            }
                            if self.workspace.config.project.freestanding == FreeStanding::True {
                                cmd.arg("-freestanding");
                            }
                            let status = cmd
                                .status()
                                .map_err(|e| format!("error: failed to invoke unnc: {}", e))?;
                            if !status.success() {
                                return Err(format!(
                                    "error: stub generation failed for {}",
                                    module_name_dep
                                ));
                            }
                        }
                        stub_paths.push(stub);
                    }
                    None => {
                        // external module already resolved
                        if let Some(dep) = resolver.resolved.get(module_name_dep) {
                            stub_paths.extend(dep.stubs.clone());
                        }
                    }
                }
            }

            let file_stem = std::path::Path::new(file)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let obj = format!("{}/{}{}", obj_dir, file_stem, obj_ext);

            if is_dirty {
                dirty_set.insert(module_name.clone());
                println!("compiling {} -> {}", file, obj);

                let mut cmd = Command::new("unnc");
                cmd.arg(file).arg("-compile").arg(&obj);

                if let Some(t) = target {
                    cmd.arg("-target").arg(t);
                }
                if self.workspace.config.project.freestanding == FreeStanding::True {
                    cmd.arg("-freestanding");
                }
                for stub in &stub_paths {
                    cmd.arg("-load").arg(stub);
                }

                let status = cmd
                    .status()
                    .map_err(|e| format!("error: failed to invoke unnc: {}", e))?;
                if !status.success() {
                    return Err(format!("error: compilation failed for {}", file));
                }

                let hash = LockFile::hash_file(file).unwrap_or_default();
                lock.entries.insert(
                    module_name.clone(),
                    crate::lock::LockEntry {
                        name: module_name.clone(),
                        path: file.clone(),
                        hash,
                        deps: imports.clone(),
                    },
                );
            } else {
                println!("skipping {} (unchanged)", file);
            }
        }

        // generate stubs for all local modules after compilation
        println!("publishing stubs...");
        for file in &self.graph.compile_order {
            let module_name = self
                .graph
                .module_map
                .iter()
                .find(|(_, v)| *v == file)
                .map(|(k, _)| k.clone())
                .unwrap_or_else(|| {
                    std::path::Path::new(file)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });

            let stub = format!("{}/{}.stub", stubs_dir, module_name);

            if !std::path::Path::new(&stub).exists() || dirty_set.contains(&module_name) {
                println!("publishing stub for {}", module_name);
                let mut cmd = Command::new("unnc");
                cmd.arg(file).arg("-stub").arg(&stub);

                if let Some(t) = target {
                    cmd.arg("-target").arg(t);
                }
                if self.workspace.config.project.freestanding == FreeStanding::True {
                    cmd.arg("-freestanding");
                }

                let status = cmd
                    .status()
                    .map_err(|e| format!("error: failed to invoke unnc: {}", e))?;
                if !status.success() {
                    return Err(format!("error: stub generation failed for {}", module_name));
                }
            }
        }

        //If the build mode is static
        if self.workspace.config.project.mode == BuildMode::Static {
            let build_dir = &self.workspace.config.layout.build;
            let name = &self.workspace.config.project.name;
            let output = format!("{}/{}.a", build_dir, name);

            println!("archiving -> {}", output);

            // collect all objs
            let mut ar_cmd = std::process::Command::new("ar");
            ar_cmd.arg("rcs").arg(&output);

            for file in &self.graph.compile_order {
                let file_stem = std::path::Path::new(file)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let obj = format!("{}/{}{}", obj_dir, file_stem, obj_ext);
                ar_cmd.arg(&obj);
            }

            let status = ar_cmd
                .status()
                .map_err(|e| format!("error: failed to invoke ar: {}", e))?;
            if !status.success() {
                return Err("error: archiving failed".to_string());
            }
        }

        // link if executable
        if self.workspace.config.project.mode == BuildMode::Executable {
            let build_dir = &self.workspace.config.layout.build;
            let name = &self.workspace.config.project.name;
            let output = format!("{}/{}{}", build_dir, name, exe_ext);

            println!("linking -> {}", output);

            let mut cmd = Command::new("unnc");
            cmd.arg("-link-only").arg("-build").arg(&output);

            if let Some(t) = target {
                cmd.arg("-target").arg(t);
            }
            if self.workspace.config.project.freestanding == FreeStanding::True {
                cmd.arg("-freestanding");
            }
            if let Some(script) = &self.workspace.config.project.script {
                cmd.arg("-script").arg(script);
            }

            let entry_stem = std::path::Path::new(&self.workspace.entry)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let entry_obj = format!("{}/{}{}", obj_dir, entry_stem, obj_ext);
            cmd.arg("-link").arg(&entry_obj);

            for file in &self.graph.compile_order {
                if file == &self.workspace.entry {
                    continue;
                }
                let file_stem = std::path::Path::new(file)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let obj = format!("{}/{}{}", obj_dir, file_stem, obj_ext);
                cmd.arg("-link").arg(&obj);
            }

            for artifact in &dependency_objects {
                cmd.arg("-link").arg(artifact);
            }

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

        lock.save("belt.lock")?;
        println!("lock file updated");

        Ok(())
    }

    pub fn check(&self) -> Result<(), String> {
        let stubs_dir = &self.workspace.config.layout.stubs;
        let mut resolver: Resolver = Resolver::new();

        // load lock file — read only, we never save after check
        let lock = LockFile::load("belt.lock");
        let mut dirty_set: std::collections::HashSet<String> = std::collections::HashSet::new();

        // resolve all external modules before compilation starts
        for module_name in &self.graph.unresolved {
            let found = resolver.resolve_project(module_name, &self.workspace.config)?;
            if !found {
                return Err(format!(
                    "error: unknown module '{}' not found in local workspace, std, or dependencies",
                    module_name
                ));
            }
        }

        for file in &self.graph.compile_order {
            let imports = match self.graph.dependecies.get(file) {
                Some(i) => i,
                None => continue,
            };

            // get module name for this file, fallback to file stem for entry point
            let module_name = self
                .graph
                .module_map
                .iter()
                .find(|(_, v)| *v == file)
                .map(|(k, _)| k.clone())
                .unwrap_or_else(|| {
                    std::path::Path::new(file)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });

            // check if dirty
            let is_dirty = lock.is_dirty(&module_name, file, imports, &dirty_set);

            if !is_dirty {
                println!("skipping {} (unchanged)", file);
                continue;
            }

            dirty_set.insert(module_name.clone());

            let mut stub_paths: Vec<String> = Vec::new();
            for module_name_dep in imports {
                match self.graph.module_map.get(module_name_dep) {
                    Some(dep_file) => {
                        let stub = format!("{}/{}.stub", stubs_dir, module_name_dep);

                        if !std::path::Path::new(&stub).exists()
                            || dirty_set.contains(module_name_dep)
                        {
                            println!("generating stub for {}", module_name_dep);
                            let status = Command::new("unnc")
                                .arg(dep_file)
                                .arg("-stub")
                                .arg(&stub)
                                .status()
                                .map_err(|e| format!("error: failed to invoke unnc: {}", e))?;
                            if !status.success() {
                                return Err(format!(
                                    "error: stub generation failed for {}",
                                    module_name_dep
                                ));
                            }
                        }
                        stub_paths.push(stub);
                    }
                    None => {
                        // already resolved above, just grab stubs
                        if let Some(dep) = resolver.resolved.get(module_name_dep) {
                            stub_paths.extend(dep.stubs.clone());
                        }
                    }
                }
            }

            // run check on this file
            let mut cmd = Command::new("unnc");
            cmd.arg(file).arg("-check");

            if self.workspace.config.project.freestanding == FreeStanding::True {
                cmd.arg("-freestanding");
            }

            if let Some(t) = &self.workspace.config.project.target {
                cmd.arg("-target").arg(t);
            }

            for stub in &stub_paths {
                cmd.arg("-load").arg(stub);
            }

            let status = cmd
                .status()
                .map_err(|e| format!("error: failed to invoke unnc: {}", e))?;

            if !status.success() {
                return Err(format!("error: check failed for {}", file));
            }
        }

        // no lock save — check produces no artifacts
        Ok(())
    }
}
