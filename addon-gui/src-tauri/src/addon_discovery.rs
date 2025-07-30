use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::addon_meta::SubAddon;

/// Removes known WoW version suffixes from .toc filenames, normalizing to "Name.toc".
///
/// https://wowpedia.fandom.com/wiki/TOC_format
/// Classic and retail versions of the game can be properly supported by including multiple TOC files in the same addon.
/// The client first searches for the respective suffix and otherwise falls back to AddonName.toc
///
/// _MainLine, _Cata, _Wrath, _TBC, _Vanilla,
/// -Cata, -WOTLKC, -BCC, -Classic
/// extra: _wotlk
///
/// # Examples
///
/// ```
/// use addon_gui_lib::addon_discovery::remove_client_flavor_toc_suffixes;
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_Mainline.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_Cata.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_Wrath.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_TBC.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_Vanilla.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_Cata.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags-WOTLKC.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_BCC.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags-Classic.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("AdiBags_wotlk.toc"), "AdiBags.toc");
/// assert_eq!(remove_client_flavor_toc_suffixes("Questie-335.toc"), "Questie-335.toc");
/// ```
pub fn remove_client_flavor_toc_suffixes(name: &str) -> String {
    let suffixes = [
        "mainline.toc",
        "cataclysm.toc",
        "cata.toc",
        "wrath.toc",
        "tbc.toc",
        "vanilla.toc",
        "classic.toc",
        "bcc.toc",
        "wotlkc.toc",
        "wotlk.toc",
    ];
    let name_lower = name.to_ascii_lowercase();
    for suf in &suffixes {
        let dash_pattern = format!("-{suf}");
        let underscore_pattern = format!("_{suf}");
        if name_lower.ends_with(&dash_pattern) {
            let idx = name_lower.rfind(&dash_pattern).unwrap();
            return name[..idx].to_string() + ".toc";
        }
        if name_lower.ends_with(&underscore_pattern) {
            let idx = name_lower.rfind(&underscore_pattern).unwrap();
            return name[..idx].to_string() + ".toc";
        }
    }
    name.to_string()
}

/// Finds all sub-addons by searching for .toc files in the root directory and immediate subdirectories only.
///
/// This function does NOT recursively walk all subdirectories to find .toc files.
/// It checks for .toc files in the root of the given path and in each immediate subdirectory (one level deep).
pub fn find_all_sub_addons(path: &PathBuf) -> Result<Vec<SubAddon>, String> {
    let mut sub_addons = Vec::new();

    // Helper to process a directory and collect .toc files
    fn collect_toc_files(dir: &Path) -> Result<Vec<String>, String> {
        let let_toc_files = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read dir: {e}"))?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_file() && path.extension() == Some(OsStr::new("toc")) {
                    path.file_name().map(|f| f.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(let_toc_files)
    }

    // Process root directory
    let toc_files = collect_toc_files(path)?;
    if !toc_files.is_empty() {
        let names = toc_files
            .iter()
            .map(|toc| remove_client_flavor_toc_suffixes(toc))
            .collect();
        sub_addons.push(SubAddon {
            dir: ".".to_string(),
            toc_files,
            names,
            enabled: true,
        });
    }

    // Process immediate subdirectories
    sub_addons.extend(
        std::fs::read_dir(path)
            .map_err(|e| format!("Failed to read repo dir: {e}"))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|sub_path| sub_path.is_dir())
            .filter_map(|sub_path| {
                let toc_files = collect_toc_files(&sub_path).ok()?;
                if toc_files.is_empty() {
                    return None;
                }
                let names = toc_files
                    .iter()
                    .map(|toc| remove_client_flavor_toc_suffixes(toc))
                    .collect();
                let dir_name = sub_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                Some(SubAddon {
                    dir: dir_name,
                    toc_files,
                    names,
                    enabled: true,
                })
            }),
    );
    Ok(sub_addons)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    /// https://github.com/Sattva-108/AdiBags
    fn test_find_all_sub_addons_single_toc_in_root() {
        let temp = tempdir().unwrap();
        let repo_dir = temp.path();

        let toc_path = repo_dir.join("AdiBags.toc");
        std::fs::File::create(&toc_path).unwrap();

        let sub_addons = find_all_sub_addons(&repo_dir.to_path_buf()).unwrap();

        assert!(
            sub_addons.len() == 1,
            "Expected 1 sub_addon, found: {:?}",
            sub_addons
        );
        assert!(
            sub_addons[0].dir == ".",
            "Expected sub_addon dir to be '.', found: {}",
            sub_addons[0].dir
        );
        assert!(
            sub_addons[0].toc_files.len() == 1,
            "Expected 1 .toc file, found: {:?}",
            sub_addons[0].toc_files
        );
        assert!(
            sub_addons[0].names.len() == 1,
            "Expected 1 name, found: {:?}",
            sub_addons[0].names
        );
        assert!(
            sub_addons[0].names[0].contains(&"AdiBags".to_string()),
            "Expected sub_addon names to contain 'AdiBags', found: {:?}",
            sub_addons[0].names[0]
        );
    }

    #[test]
    /// https://github.com/Sattva-108/AdiBags-WoTLK-3.3.5-Mods
    fn test_find_all_sub_addons_multiple_subdirs_with_toc() {
        let temp = tempdir().unwrap();
        let repo_dir = temp.path();

        // Create subdirectories
        let sub1 = repo_dir.join("AdiBags-ItemOverlayPlus");
        let sub2 = repo_dir.join("AdiBags_Bound");
        let sub3 = repo_dir.join("NoTocSubAddon");
        std::fs::create_dir_all(&sub1).unwrap();
        std::fs::create_dir_all(&sub2).unwrap();
        std::fs::create_dir_all(&sub3).unwrap();

        // Create .toc files in each sub-addon directory
        let toc1 = sub1.join("AdiBags-ItemOverlayPlus.toc");
        let toc2 = sub2.join("AdiBags_Bound.toc");
        std::fs::File::create(&toc1).unwrap();
        std::fs::File::create(&toc2).unwrap();

        let sub_addons = find_all_sub_addons(&repo_dir.to_path_buf()).unwrap();

        assert_eq!(
            sub_addons.len(),
            2,
            "Expected 2 sub_addons, found: {:?}",
            sub_addons
        );

        let mut found_dirs = sub_addons.iter().map(|s| s.dir.clone()).collect::<Vec<_>>();
        found_dirs.sort();
        assert_eq!(
            found_dirs,
            vec!["AdiBags-ItemOverlayPlus", "AdiBags_Bound"],
            "Unexpected sub_addon dirs: {:?}",
            found_dirs
        );

        for sub in &sub_addons {
            assert_eq!(
                sub.toc_files.len(),
                1,
                "Expected 1 .toc file in {:?}, found: {:?}",
                sub.dir,
                sub.toc_files
            );
            assert_eq!(
                sub.names.len(),
                1,
                "Expected 1 name in {:?}, found: {:?}",
                sub.dir,
                sub.names
            );
            assert!(
                sub.toc_files[0].ends_with(".toc"),
                "Expected .toc file, found: {:?}",
                sub.toc_files[0]
            );
        }
    }

    #[test]
    /// https://github.com/widxwer/Questie
    /// This Questie has multiple basename .toc files in the root directory
    /// It is expected that the user renames the folder manually
    /// We should discover the multiple base names
    fn test_find_all_sub_addons_questie_multiple_tocs_in_root() {
        let temp = tempdir().unwrap();
        let repo_dir = temp.path();

        // Create multiple .toc files in the root directory
        let toc_files = vec![
            "Questie-335-Classic.toc",
            "Questie-335-TBC.toc",
            "Questie-335.toc",
            "Questie-BCC.toc",
            "Questie-Classic.toc",
            "Questie-WOTLKC.toc",
            "Questie.toc",
        ];
        for toc in &toc_files {
            std::fs::File::create(repo_dir.join(toc)).unwrap();
        }

        let sub_addons = find_all_sub_addons(&repo_dir.to_path_buf()).unwrap();

        assert_eq!(
            sub_addons.len(),
            1,
            "Expected 1 sub_addon, found: {:?}",
            sub_addons
        );
        let sub = &sub_addons[0];
        assert_eq!(
            sub.dir, ".",
            "Expected sub_addon dir to be '.', found: {}",
            sub.dir
        );
        assert_eq!(
            sub.toc_files.len(),
            toc_files.len(),
            "Expected all .toc files to be detected, found: {:?}",
            sub.toc_files
        );

        for toc in &toc_files {
            assert!(
                sub.toc_files.contains(&toc.to_string()),
                "Missing toc file: {}",
                toc
            );
        }

        for name in &sub.names {
            assert!(
                name == "Questie.toc" || name == "Questie-335.toc",
                "Expected normalized name to be 'Questie.toc' or 'Questie-335.toc', found: {}",
                name
            );
        }
    }
}
