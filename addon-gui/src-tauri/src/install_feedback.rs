use crate::addon_meta::AddonMeta;
use std::path::Path;

/// Like install_sub_addons, but with feedback callbacks
pub fn install_sub_addons_with_feedback<W, E, S>(
    mut addon_meta: AddonMeta,
    repo_root: &Path,
    addons_dir: &Path,
    warning: &mut W,
    error: &mut E,
    status: &mut S,
) where
    W: FnMut(&str),
    E: FnMut(&str),
    S: FnMut(&str),
{
    let sub_addons = &addon_meta.sub_addons;

    for sub in sub_addons {
        if !sub.enabled {
            continue;
        }
        let symlink_name = &sub.name;
        let target_dir = if sub.dir == "." {
            repo_root.to_path_buf()
        } else {
            repo_root.join(&sub.dir)
        };
        let symlink_path = addons_dir.join(symlink_name);

        if symlink_path.exists() {
            status(&format!("Removing existing symlink or directory: {}", symlink_path.display()));
            std::fs::remove_file(&symlink_path)
                .or_else(|_| std::fs::remove_dir_all(&symlink_path))
                .ok();
        }

        if sub.names.len() > 1 {
            warning(&format!(
                "Multiple possible names for sub-addon '{}': {:?}. Using '{}'.",
                sub.dir, sub.names, symlink_name
            ));
        }

        status(&format!("Creating symlink for '{}': {} -> {}", symlink_name, target_dir.display(), symlink_path.display()));
        if let Err(e) = crate::symlink::create_symlink(&target_dir, &symlink_path) {
            error(&format!(
                "Failed to create symlink for '{symlink_name}': {} -> {} ({e})",
                target_dir.display(),
                symlink_path.display(),
            ));
        }
    }
    addon_meta.installed_at = Some(chrono::Utc::now().to_rfc3339());
}
