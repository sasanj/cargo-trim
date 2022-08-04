use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::utils::get_size;

#[derive(Debug)]
pub(crate) struct CrateInfo {
    size: u64,
}

impl CrateInfo {
    pub(crate) fn size(&self) -> u64 {
        self.size
    }
}

impl PartialEq for CrateInfo {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
    }
}

/// stores different crate size and name information
#[derive(Default)]
pub(crate) struct CrateDetail {
    bin: HashMap<String, CrateInfo>,
    git_crates_source: HashMap<String, CrateInfo>,
    registry_crates_source: HashMap<String, CrateInfo>,
    git_crates_archive: HashMap<String, CrateInfo>,
    registry_crates_archive: HashMap<String, CrateInfo>,
}

impl CrateDetail {
    /// return bin crates size information
    pub(crate) fn bin(&self) -> &HashMap<String, CrateInfo> {
        &self.bin
    }

    /// return git crates source size information
    pub(crate) fn git_crates_source(&self) -> &HashMap<String, CrateInfo> {
        &self.git_crates_source
    }

    /// return registry crates source size information
    pub(crate) fn registry_crates_source(&self) -> &HashMap<String, CrateInfo> {
        &self.registry_crates_source
    }

    /// return git crates archive size information
    pub(crate) fn git_crates_archive(&self) -> &HashMap<String, CrateInfo> {
        &self.git_crates_archive
    }

    /// return registry crates archive size information
    pub(crate) fn registry_crates_archive(&self) -> &HashMap<String, CrateInfo> {
        &self.registry_crates_archive
    }

    /// add bin information to crate detail
    fn add_bin(&mut self, bin_name: String, size: u64) {
        self.bin.insert(bin_name, CrateInfo { size });
    }

    /// add git crate source information to crate detail
    fn add_git_crate_source(&mut self, crate_name: String, size: u64) {
        add_crate_to_hash_map(&mut self.git_crates_source, crate_name, size);
    }

    /// add registry crate source information to crate detail
    fn add_registry_crate_source(&mut self, crate_name: String, size: u64) {
        add_crate_to_hash_map(&mut self.registry_crates_source, crate_name, size);
    }

    /// add git crate archive information to crate detail
    fn add_git_crate_archive(&mut self, crate_name: String, size: u64) {
        add_crate_to_hash_map(&mut self.git_crates_archive, crate_name, size);
    }

    /// add registry crate archive information to crate detail
    fn add_registry_crate_archive(&mut self, crate_name: String, size: u64) {
        add_crate_to_hash_map(&mut self.registry_crates_archive, crate_name, size);
    }

    /// find size of certain git crate source in KB
    fn find_size_git_source(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.git_crates_source, crate_name)
    }

    /// find size of certain registry source in KB
    fn find_size_registry_source(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.registry_crates_source, crate_name)
    }

    /// find size of certain git crate archive in KB
    fn find_size_git_archive(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.git_crates_archive, crate_name)
    }

    /// find size of certain registry archive in KB
    fn find_size_registry_archive(&self, crate_name: &str) -> f64 {
        get_hashmap_crate_size(&self.registry_crates_archive, crate_name)
    }

    /// return certain git crate total size in KB
    pub(crate) fn find_size_git_all(&self, crate_name: &str) -> f64 {
        self.find_size_git_archive(crate_name) + self.find_size_git_source(crate_name)
    }

    /// return certain registry crate total size in KB
    pub(crate) fn find_size_registry_all(&self, crate_name: &str) -> f64 {
        self.find_size_registry_archive(crate_name) + self.find_size_registry_source(crate_name)
    }

    /// find crate size if location/title is given in KB
    pub(crate) fn find(&self, crate_name: &str, location: &str) -> f64 {
        if location.contains("REGISTRY") {
            self.find_size_registry_all(crate_name)
        } else if location.contains("GIT") {
            self.find_size_git_all(crate_name)
        } else {
            0.0
        }
    }

    /// list installed bin
    pub(crate) fn list_installed_bin(&mut self, bin_dir: &Path) -> Result<Vec<String>> {
        let mut installed_bin = Vec::new();
        if bin_dir.exists() {
            for entry in fs::read_dir(bin_dir).context("failed to read bin directory")? {
                let entry = entry?.path();
                let bin_size = get_size(&entry).context("failed to get size of bin directory")?;
                let file_name = entry
                    .file_name()
                    .context("failed to get file name from bin directory")?;
                let bin_name = file_name.to_str().unwrap().to_string();
                self.add_bin(bin_name.clone(), bin_size);
                installed_bin.push(bin_name);
            }
        }
        installed_bin.sort();
        Ok(installed_bin)
    }

    /// list all installed registry crates
    pub(crate) fn list_installed_crate_registry(
        &mut self,
        src_dir: &Path,
        cache_dir: &Path,
    ) -> Result<Vec<String>> {
        let mut installed_crate_registry = Vec::new();
        // read src dir to get installed crate
        if src_dir.exists() {
            for entry in fs::read_dir(src_dir).context("failed to read src directory")? {
                let registry = entry?.path();
                for entry in fs::read_dir(registry).context("failed to read registry folder")? {
                    let entry = entry?.path();
                    let crate_size =
                        get_size(&entry).context("failed to get registry crate size")?;
                    let file_name = entry
                        .file_name()
                        .context("failed to get file name form main entry")?;
                    let crate_name = file_name.to_str().unwrap();
                    self.add_registry_crate_source(crate_name.to_owned(), crate_size);
                    installed_crate_registry.push(crate_name.to_owned());
                }
            }
        }
        // read cache dir to get installed crate
        if cache_dir.exists() {
            for entry in fs::read_dir(cache_dir).context("failed to read cache dir")? {
                let registry = entry?.path();
                for entry in
                    fs::read_dir(registry).context("failed to read cache dir registry folder")?
                {
                    let entry = entry?.path();
                    let file_name = entry
                        .file_name()
                        .context("failed to get file name from cache dir")?;
                    let crate_size = get_size(&entry).context("failed to get size")?;
                    let crate_name = file_name.to_str().unwrap();
                    let split_name = crate_name.rsplitn(2, '.').collect::<Vec<&str>>();
                    self.add_registry_crate_archive(split_name[1].to_owned(), crate_size);
                    installed_crate_registry.push(split_name[1].to_owned());
                }
            }
        }
        installed_crate_registry.sort();
        installed_crate_registry.dedup();
        Ok(installed_crate_registry)
    }

    /// list all installed git crates
    pub(crate) fn list_installed_crate_git(
        &mut self,
        checkout_dir: &Path,
        db_dir: &Path,
    ) -> Result<Vec<String>> {
        let mut installed_crate_git = Vec::new();
        if checkout_dir.exists() {
            // read checkout dir to list crate name in form of crate_name-rev_sha
            for entry in fs::read_dir(checkout_dir).context("failed to read checkout directory")? {
                let entry = entry?.path();
                let file_path = entry
                    .file_name()
                    .context("failed to obtain checkout directory sub folder file name")?;
                for git_sha_entry in
                    fs::read_dir(&entry).context("failed to read checkout dir sub folder")?
                {
                    let git_sha_entry = git_sha_entry?.path();
                    let crate_size =
                        get_size(&git_sha_entry).context("failed to get folder size")?;
                    let git_sha_file_name = git_sha_entry
                        .file_name()
                        .context("failed to get file name")?;
                    let git_sha = git_sha_file_name.to_str().unwrap();
                    let file_name = file_path.to_str().unwrap();
                    let split_name = file_name.rsplitn(2, '-').collect::<Vec<&str>>();
                    let full_name = format!("{}-{}", split_name[1], git_sha);
                    self.add_git_crate_archive(full_name.clone(), crate_size);
                    installed_crate_git.push(full_name);
                }
            }
        }
        // read a database directory to list a git crate in form of crate_name-HEAD
        if db_dir.exists() {
            for entry in fs::read_dir(db_dir).context("failed to read db dir")? {
                let entry = entry?.path();
                let crate_size =
                    get_size(&entry).context("failed to get size of db dir folders")?;
                let file_name = entry.file_name().context("failed to get file name")?;
                let file_name = file_name.to_str().unwrap();
                let split_name = file_name.rsplitn(2, '-').collect::<Vec<&str>>();
                let full_name = format!("{}-HEAD", split_name[1]);
                self.add_git_crate_source(full_name.clone(), crate_size);
                installed_crate_git.push(full_name);
            }
        }
        installed_crate_git.sort();
        installed_crate_git.dedup();
        Ok(installed_crate_git)
    }
}

/// Convert stored bytes size to KB and return f64 for crate from hashmap
#[allow(clippy::cast_precision_loss)]
fn get_hashmap_crate_size(hashmap: &HashMap<String, CrateInfo>, crate_name: &str) -> f64 {
    hashmap
        .get(crate_name)
        .map_or(0.0, |info| (info.size as f64) / 1000_f64.powi(2))
}

fn add_crate_to_hash_map(hashmap: &mut HashMap<String, CrateInfo>, crate_name: String, size: u64) {
    if let Some(info) = hashmap.get_mut(&crate_name) {
        info.size += size;
    } else {
        hashmap.insert(crate_name, CrateInfo { size });
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::{add_crate_to_hash_map, get_hashmap_crate_size};
    use crate::crate_detail::CrateInfo;
    #[test]
    fn test_get_hashmap_crate_size() {
        let mut hashmap_content = HashMap::new();
        hashmap_content.insert("sample_crate".to_string(), CrateInfo { size: 1000 });
        hashmap_content.insert("sample_crate_2".to_string(), CrateInfo { size: 20 });

        assert_eq!(
            get_hashmap_crate_size(&hashmap_content, "sample_crate_2"),
            0.00002
        );
        assert_eq!(
            get_hashmap_crate_size(&hashmap_content, "sample_crate_3"),
            0.0
        );
    }
    #[test]
    fn test_add_crate_to_hashmap() {
        let mut hashmap_content = HashMap::new();
        hashmap_content.insert("sample_crate".to_string(), CrateInfo { size: 10000 });
        hashmap_content.insert("sample_crate_2".to_string(), CrateInfo { size: 20 });
        add_crate_to_hash_map(&mut hashmap_content, "sample_crate_2".to_string(), 3000);
        add_crate_to_hash_map(&mut hashmap_content, "sample_crate_3".to_string(), 2500);

        let mut another_hashmap = HashMap::new();
        another_hashmap.insert("sample_crate".to_string(), CrateInfo { size: 10000 });
        another_hashmap.insert("sample_crate_2".to_string(), CrateInfo { size: 3020 });
        another_hashmap.insert("sample_crate_3".to_string(), CrateInfo { size: 2500 });

        assert_eq!(hashmap_content, another_hashmap);
    }
}
