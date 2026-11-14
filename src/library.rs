use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub struct Library {
    pub root: PathBuf,
    _lock: File,
}
impl Library {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        if !path.is_absolute() {
            return Err("Library must be an absolute path outside Git.".into());
        }
        check_path(&path)?;
        fs::create_dir_all(&path)
            .map_err(|_| "Cannot create library. Choose a writable external directory.")?;
        check_path(&path)?;
        let root = fs::canonicalize(path).map_err(|_| "Cannot resolve library directory.")?;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .map_err(|_| "Cannot set private library permissions.")?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(root.join(".lock"))
            .map_err(|_| "Cannot lock library.")?;
        lock.try_lock_exclusive()
            .map_err(|_| "Library is already open in another Studio process.")?;
        let this = Self { root, _lock: lock };
        this.preflight()?;
        Ok(this)
    }
    pub fn preflight(&self) -> Result<(), String> {
        check_path(&self.root)?;
        if fs2::available_space(&self.root).map_err(|_| "Cannot inspect library space.")?
            < 256 * 1024 * 1024
        {
            return Err("Library needs at least 256 MiB of free space before generation.".into());
        }
        self.write(".write-check", b"writable")?;
        fs::remove_file(self.root.join(".write-check")).map_err(|_| "Library is not writable.")?;
        Ok(())
    }
    pub fn write(&self, name: &str, bytes: &[u8]) -> Result<(), String> {
        check_path(&self.root)?;
        if name.contains('/') || name.contains('\\') {
            return Err("Invalid library identity.".into());
        }
        let destination = self.root.join(name);
        if fs::symlink_metadata(&destination).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err("Library contains an unsafe symbolic link.".into());
        }
        let temp = self.root.join(format!(".{}.tmp", Uuid::new_v4()));
        let result = (|| {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(&temp)?;
            f.write_all(bytes)?;
            f.sync_all()?;
            fs::rename(&temp, destination)?;
            File::open(&self.root)?.sync_all()
        })();
        if result.is_err() {
            let _ = fs::remove_file(temp);
        }
        result.map_err(|_: std::io::Error| {
            "Local save failed. Keep Studio open and retry saving; do not regenerate.".into()
        })
    }
    pub fn read(&self, name: &str, max: u64) -> Result<Vec<u8>, String> {
        check_path(&self.root)?;
        if name.contains('/') || name.contains('\\') {
            return Err("Invalid library identity.".into());
        }
        let mut f = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(self.root.join(name))
            .map_err(|_| "Library file unavailable or unsafe.")?;
        let m = f.metadata().map_err(|_| "Cannot inspect library file.")?;
        if !m.is_file() || m.len() > max {
            return Err("Library file exceeds allowed size or is not regular.".into());
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut f)
            .take(max + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Cannot read library file.")?;
        if bytes.len() as u64 > max {
            return Err("Library file exceeds allowed size.".into());
        }
        Ok(bytes)
    }
    pub fn recipes(&self) -> Result<Vec<String>, String> {
        check_path(&self.root)?;
        let mut names = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(|_| "Cannot list library.")? {
            let entry = entry.map_err(|_| "Cannot read library entry.")?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(id) = name.strip_suffix(".recipe.json")
                && Uuid::parse_str(id).is_ok()
            {
                names.push(name);
            }
        }
        Ok(names)
    }
}
fn check_path(path: &Path) -> Result<(), String> {
    let mut partial = PathBuf::new();
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err("Library path cannot contain parent traversal.".into());
        }
        partial.push(component);
        if fs::symlink_metadata(&partial).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err(
                "Library path cannot contain symbolic links. Choose its real external location."
                    .into(),
            );
        }
        if partial.join(".git").exists() {
            return Err(
                "Runtime library cannot be inside a Git working tree (including linked worktrees)."
                    .into(),
            );
        }
    }
    Ok(())
}
