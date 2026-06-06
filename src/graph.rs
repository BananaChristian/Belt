use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;

fn topological_sort(
    file: &str,
    graph: &HashMap<String, Vec<String>>,
    module_map: &HashMap<String, String>,
    visited: &mut HashSet<String>,
    visiting: &mut HashSet<String>,
    order: &mut Vec<String>,
) -> Result<(), String> {
    if visiting.contains(file) {
        return Err(format!(
            "error: circular import detected involving {}",
            file
        ));
    }
    if visited.contains(file) {
        return Ok(());
    }

    visiting.insert(file.to_string());

    if let Some(imports) = graph.get(file) {
        for module_name in imports {
            match module_map.get(module_name) {
                Some(dep_file) => {
                    topological_sort(dep_file, graph, module_map, visited, visiting, order)?
                }
                None => {
                    continue;
                }
            }
        }
    }

    visiting.remove(file);
    visited.insert(file.to_string());
    order.push(file.to_string());

    Ok(())
}

pub fn get_module_name(file_path: &str) -> Option<String> {
    let content = fs::read_to_string(file_path).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("module") {
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
            if parts.len() == 2 {
                return Some(parts[1].trim().to_string());
            }
        } else if !trimmed.is_empty() {
            break; // module declaration must be at top
        }
    }
    None
}

pub fn build_module_map(source_files: &Vec<String>) -> Result<HashMap<String, String>, String> {
    let mut map: HashMap<String, String> = HashMap::new();

    for file in source_files {
        if let Some(module_name) = get_module_name(file) {
            if map.contains_key(&module_name) {
                return Err(format!(
                    "error: duplicate module name '{}' found in {}",
                    module_name, file
                ));
            }
            map.insert(module_name, file.clone());
        }
    }

    Ok(map)
}

pub fn get_imports(file_path: &str) -> Vec<String> {
    let mut imports = Vec::new();
    let contents = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return imports,
    };

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import") {
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
            if parts.len() == 2 {
                imports.push(parts[1].trim().to_string());
            }
        } else if trimmed.starts_with("module") {
            continue;
        } else if !trimmed.is_empty() {
            break;
        }
    }

    imports
}

pub struct Graph {
    pub module_map: HashMap<String, String>, //module_name => file_path
    pub dependecies: HashMap<String, Vec<String>>, //file_path => [files it imports]
    pub compile_order: Vec<String>,          //file_paths in compile order
    pub unresolved: Vec<String>,             // modules not found local
}

impl Graph {
    pub fn build(source_files: &Vec<String>) -> Result<Graph, String> {
        let mut graph = Graph {
            module_map: HashMap::new(),
            dependecies: HashMap::new(),
            compile_order: Vec::new(),
            unresolved: Vec::new(),
        };

        graph.module_map = build_module_map(source_files)?;

        // First populate dependencies
        for file in source_files {
            let imports = get_imports(file);
            graph.dependecies.insert(file.clone(), imports);
        }

        // Then do topological sort
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        for file in source_files {
            topological_sort(
                file,
                &graph.dependecies,
                &graph.module_map,
                &mut visited,
                &mut visiting,
                &mut graph.compile_order,
            )?;
        }

        // Then collect unresolved
        for file in source_files {
            if let Some(imports) = graph.dependecies.get(file) {
                for module_name in imports {
                    if !graph.module_map.contains_key(module_name)
                        && !graph.unresolved.contains(module_name)
                    {
                        graph.unresolved.push(module_name.clone());
                    }
                }
            }
        }

        Ok(graph)
    }
}
