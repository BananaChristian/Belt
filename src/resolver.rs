use crate::config::{self, Config, FreeStanding};
use crate::load_external_config;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct ResolvedDependecy {
    pub stubs: Vec<String>,
    pub artifacts: Vec<String>, //What we wanna link
}

pub struct Resolver {
    pub resolved: HashMap<String, ResolvedDependecy>,
    visited_projects: HashSet<PathBuf>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            resolved: HashMap::new(),
            visited_projects: HashSet::new(),
        }
    }

    fn compiler_root() -> Option<PathBuf> {
        let unnc_name = if cfg!(windows) { "unnc.exe" } else { "unnc" };
        std::env::var_os("PATH")
            .and_then(|paths| {
                std::env::split_paths(&paths)
                    .filter_map(|dir| {
                        let candidate = dir.join(unnc_name);
                        if candidate.exists() {
                            Some(candidate)
                        } else {
                            None
                        }
                    })
                    .next()
            })
            .and_then(|unnc_path| {
                unnc_path
                    .parent()? //bin/
                    .parent() //root
                    .map(|p| p.to_path_buf())
            })
    }

    fn find_artifacts(dep_root: &Path, config: &Config) -> Vec<String> {
        let mut artifacts = Vec::new();
        // .o files live in objs/
        let objs_dir = dep_root.join(&config.layout.objs);
        if let Ok(entries) = std::fs::read_dir(&objs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "o" || ext == "obj" {
                        artifacts.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // .a and .so live in build/
        let build_dir = dep_root.join(&config.layout.build);
        if let Ok(entries) = std::fs::read_dir(&build_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "a" || ext == "so" {
                        artifacts.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        artifacts
    }

    fn find_stub(stubs_dir: &Path, module_name: &str) -> Option<String> {
        let stub = stubs_dir.join(format!("{}.stub", module_name));
        if stub.exists() {
            Some(stub.to_string_lossy().to_string())
        } else {
            None
        }
    }

    fn resolve_dependencies(&mut self, path: &Path, module_name: &str) -> Result<bool, String> {
        let canonical = path.canonicalize().unwrap_or(path.to_path_buf());
        if self.visited_projects.contains(&canonical) {
            return Ok(false);
        }
        self.visited_projects.insert(canonical);

        let config = load_external_config(path)?;
        let stub_dir = path.join(&config.layout.stubs);
        let build_dir = path.join(&config.layout.build);

        if let Some(stub) = Self::find_stub(&stub_dir, module_name) {
            self.resolved.insert(
                module_name.to_string(),
                ResolvedDependecy {
                    stubs: vec![stub],
                    artifacts: Self::find_artifacts(&build_dir,&config),
                },
            );
            return Ok(true);
        }

        // check transitive deps
        if let Some(deps) = &config.dependecies.clone() {
            for (_name, dep_path) in &deps.deps {
                let dep_root = PathBuf::from(dep_path);
                if self.resolve_dependencies(&dep_root, module_name)? {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn resolve_project(
        &mut self,
        module_name: &str,
        current_config: &Config,
    ) -> Result<bool, String> {
        // already resolved
        if self.resolved.contains_key(module_name) {
            return Ok(true);
        }

        // Stage 2 std
        if current_config.project.freestanding != FreeStanding::True {
            if let Some(root) = Self::compiler_root() {
                let std_dir = root.join("std");
                if self.resolve_dependencies(std_dir.as_path(), module_name)? {
                    return Ok(true);
                }
            }
        }

        // Stage 3 user deps
        if let Some(deps) = &current_config.dependecies.clone() {
            for (_name, path) in &deps.deps {
                let dep_root = PathBuf::from(path);
                if self.resolve_dependencies(&dep_root, module_name)? {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
