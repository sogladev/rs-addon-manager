use std::path::Path;

#[cfg(target_family = "unix")]
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

#[cfg(target_family = "windows")]
pub fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> std::io::Result<()> {
    junction::create(src.as_ref(), dst.as_ref())
}
