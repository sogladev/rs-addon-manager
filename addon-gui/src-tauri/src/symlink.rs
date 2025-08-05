use std::path::Path;

#[cfg(target_family = "unix")]
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(target_family = "windows")]
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> std::io::Result<()> {
    junction::create(src.as_ref(), dst.as_ref())
}

/// Check if a path is a symlink (Unix) or junction (Windows)
pub fn is_addon_symlinked<P: AsRef<Path>>(path: P) -> bool {
    let p = path.as_ref();
    #[cfg(target_family = "windows")]
    {
        junction::exists(p).unwrap_or(false)
    }
    #[cfg(not(target_family = "windows"))]
    {
        p.exists() && p.is_symlink()
    }
}
