use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;

pub struct LockEntry {
    pub name: String,
    pub path: String,
    pub hash: String,
    pub deps: Vec<String>,
}

pub struct LockFile {
    pub entries: HashMap<String, LockEntry>,
}

impl LockFile {
    // Create empty lock file
    pub fn new() -> Self {
        LockFile {
            entries: HashMap::new(),
        }
    }

    // Load from disk, returns empty if not found
    pub fn load(path: &str) -> Self {
        let mut lock = LockFile::new();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return lock, // no lock file yet, fresh build
        };

        for line in content.lines() {
            let parts: Vec<&str> = line.splitn(5, ' ').collect();
            if parts.len() < 4 || parts[0] != "module" {
                continue;
            }
            let name = parts[1].to_string();
            let path = parts[2].to_string();
            let hash = parts[3].to_string();
            let deps = if parts.len() == 5 {
                parts[4].split_whitespace()
                    .map(|s| s.to_string())
                    .collect()
            } else {
                Vec::new()
            };

            lock.entries.insert(name.clone(), LockEntry { name, path, hash, deps });
        }

        lock
    }

    // Save to disk
    pub fn save(&self, path: &str) -> Result<(), String> {
        let mut file = fs::File::create(path)
            .map_err(|e| format!("error: failed to write lock file: {}", e))?;

        for entry in self.entries.values() {
            let deps_str = entry.deps.join(" ");
            let line = if deps_str.is_empty() {
                format!("module {} {} {}\n", entry.name, entry.path, entry.hash)
            } else {
                format!("module {} {} {} {}\n", entry.name, entry.path, entry.hash, deps_str)
            };
            file.write_all(line.as_bytes())
                .map_err(|e| format!("error: failed to write lock file: {}", e))?;
        }

        Ok(())
    }

    // Hash a file's contents
    pub fn hash_file(path: &str) -> Option<String> {
        let content = fs::read(path).ok()?;
        // simple hash using std — no external crates
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Some(format!("{:x}", hasher.finish()))
    }

    // Check if a module is dirty
    pub fn is_dirty(
        &self,
        module_name: &str,
        file_path: &str,
        deps: &[String],
        dirty_set: &HashSet<String>,
    ) -> bool {
        // not in lock file — new file, always dirty
        let entry = match self.entries.get(module_name) {
            Some(e) => e,
            None => return true,
        };

        // hash changed
        if let Some(current_hash) = Self::hash_file(file_path) {
            if current_hash != entry.hash {
                return true;
            }
        }

        // any dependency is dirty
        for dep in deps {
            if dirty_set.contains(dep) {
                return true;
            }
        }

        false
    }
}
