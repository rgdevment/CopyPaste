use crate::{Error, Result};
use std::path::{Path, PathBuf};

pub const IN_A_DIGEST: usize = blake3::OUT_LEN * 2;

pub fn is_a_digest(said: &str) -> bool {
    said.len() == IN_A_DIGEST
        && said
            .bytes()
            .all(|one| one.is_ascii_digit() || (b'a'..=b'f').contains(&one))
}

pub struct Blobs {
    root: PathBuf,
}

static WRITES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Blobs {
    pub fn at(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root).map_err(Error::Io)?;
        crate::store::restrict(root, 0o700)?;
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    fn path_of(&self, digest: &str) -> PathBuf {
        self.root
            .join(&digest[0..2])
            .join(&digest[2..4])
            .join(digest)
    }

    fn path_for(&self, digest: &str) -> Option<PathBuf> {
        is_a_digest(digest).then(|| self.path_of(digest))
    }

    pub fn put(&self, bytes: &[u8]) -> Result<String> {
        let digest = blake3::hash(bytes).to_hex().to_string();
        let path = self.path_of(&digest);
        if let Ok(file) = std::fs::File::options().write(true).open(&path) {
            file.set_modified(std::time::SystemTime::now())
                .map_err(Error::Io)?;
            return Ok(digest);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(Error::Io)?;
            crate::store::restrict(parent, 0o700)?;
        }
        let temporary = path.with_extension(format!(
            "{}-{}.partial",
            std::process::id(),
            WRITES.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::write(&temporary, bytes).map_err(Error::Io)?;
        crate::store::restrict(&temporary, 0o600)?;
        std::fs::rename(&temporary, &path).map_err(Error::Io)?;
        Ok(digest)
    }

    pub fn get(&self, digest: &str) -> Result<Option<Vec<u8>>> {
        let Some(path) = self.path_for(digest) else {
            return Ok(None);
        };
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(why) => Err(Error::Io(why)),
        }
    }

    pub fn remove(&self, digest: &str) -> Result<()> {
        match self.path_for(digest) {
            Some(path) => remove_at(&path),
            None => Ok(()),
        }
    }

    pub fn remove_if_settled(&self, digest: &str) -> Result<bool> {
        let Some(path) = self.path_for(digest) else {
            return Ok(false);
        };
        if !path.exists() || is_fresh(&path) {
            return Ok(false);
        }
        remove_at(&path).map(|()| true)
    }

    pub fn where_it_is(&self, digest: &str) -> Option<PathBuf> {
        self.path_for(digest)
    }

    pub fn exists(&self, digest: &str) -> bool {
        self.path_for(digest).is_some_and(|path| path.exists())
    }

    pub const GRACE: std::time::Duration = std::time::Duration::from_secs(60);

    pub fn sweep(&self, referenced: &dyn Fn(&str) -> bool) -> Result<usize> {
        let mut removed = 0;
        for path in files_under(&self.root) {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Some((digest, partial)) = digest_of(name) else {
                continue;
            };
            if is_fresh(&path) || (!partial && referenced(digest)) {
                continue;
            }
            remove_at(&path)?;
            removed += 1;
        }
        Ok(removed)
    }
}

fn is_fresh(path: &Path) -> bool {
    let settled = std::time::SystemTime::now() - Blobs::GRACE;
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .is_ok_and(|modified| modified >= settled)
}

fn digest_of(name: &str) -> Option<(&str, bool)> {
    let (digest, partial) = match name.strip_suffix(".partial") {
        Some(rest) => (rest.split('.').next().unwrap_or(rest), true),
        None => (name, false),
    };
    is_a_digest(digest).then_some((digest, partial))
}

const AT_A_TIME: usize = 64 * 1024;

pub fn remove_at(path: &Path) -> Result<()> {
    use std::io::Write;
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.file_type().is_symlink() {
        std::fs::remove_file(path).map_err(Error::Io)?;
        return Ok(());
    }
    if !metadata.is_file() {
        return Ok(());
    }
    let mut file = std::fs::File::options()
        .write(true)
        .open(path)
        .map_err(Error::Io)?;
    let zeros = [0u8; AT_A_TIME];
    let mut left = metadata.len();
    while left > 0 {
        let now = usize::try_from(left).unwrap_or(AT_A_TIME).min(AT_A_TIME);
        file.write_all(&zeros[..now]).map_err(Error::Io)?;
        left -= now as u64;
    }
    file.sync_all().map_err(Error::Io)?;
    drop(file);
    std::fs::remove_file(path).map_err(Error::Io)?;
    Ok(())
}

pub(crate) fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            found.extend(files_under(&path));
        } else {
            found.push(path);
        }
    }
    found
}

#[cfg(test)]
#[path = "blobs_test.rs"]
mod tests;
