use crate::{Error, Result};
use std::path::{Path, PathBuf};

pub struct Blobs {
    root: PathBuf,
}

impl Blobs {
    pub fn at(root: &Path) -> Result<Self> {
        std::fs::create_dir_all(root).map_err(Error::Io)?;
        crate::store::restrict(root, 0o700)?;
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    fn path_for(&self, digest: &str) -> PathBuf {
        self.root
            .join(&digest[0..2])
            .join(&digest[2..4])
            .join(digest)
    }

    pub fn put(&self, bytes: &[u8]) -> Result<String> {
        let digest = blake3::hash(bytes).to_hex().to_string();
        let path = self.path_for(&digest);
        if let Ok(file) = std::fs::File::options().write(true).open(&path) {
            file.set_modified(std::time::SystemTime::now())
                .map_err(Error::Io)?;
            return Ok(digest);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(Error::Io)?;
            crate::store::restrict(parent, 0o700)?;
        }
        let temporary = path.with_extension("partial");
        std::fs::write(&temporary, bytes).map_err(Error::Io)?;
        crate::store::restrict(&temporary, 0o600)?;
        std::fs::rename(&temporary, &path).map_err(Error::Io)?;
        Ok(digest)
    }

    pub fn get(&self, digest: &str) -> Result<Option<Vec<u8>>> {
        let path = self.path_for(digest);
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(why) => Err(Error::Io(why)),
        }
    }

    pub fn remove(&self, digest: &str) -> Result<()> {
        remove_at(&self.path_for(digest))
    }

    pub fn exists(&self, digest: &str) -> bool {
        self.path_for(digest).exists()
    }

    pub const GRACE: std::time::Duration = std::time::Duration::from_secs(60);

    pub fn sweep(&self, referenced: &dyn Fn(&str) -> bool) -> Result<usize> {
        let settled = std::time::SystemTime::now() - Self::GRACE;
        let mut removed = 0;
        for path in files_under(&self.root) {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Some((digest, partial)) = digest_of(name) else {
                continue;
            };
            let fresh = std::fs::metadata(&path)
                .and_then(|meta| meta.modified())
                .is_ok_and(|modified| modified >= settled);
            if fresh || (!partial && referenced(digest)) {
                continue;
            }
            remove_at(&path)?;
            removed += 1;
        }
        Ok(removed)
    }
}

fn digest_of(name: &str) -> Option<(&str, bool)> {
    let (digest, partial) = match name.strip_suffix(".partial") {
        Some(digest) => (digest, true),
        None => (name, false),
    };
    let shaped = digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit());
    shaped.then_some((digest, partial))
}

fn remove_at(path: &Path) -> Result<()> {
    let Ok(metadata) = std::fs::metadata(path) else {
        return Ok(());
    };
    let zeros = vec![0u8; metadata.len() as usize];
    std::fs::write(path, &zeros).map_err(Error::Io)?;
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
        if path.is_dir() {
            found.extend(files_under(&path));
        } else {
            found.push(path);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary() -> (tempfile::TempDir, Blobs) {
        let dir = tempfile::tempdir().expect("carpeta");
        let blobs = Blobs::at(&dir.path().join("blobs")).expect("almacén");
        (dir, blobs)
    }

    #[test]
    fn what_goes_in_comes_out() {
        let (_dir, blobs) = temporary();
        let digest = blobs.put(b"unos bytes cualesquiera").expect("guarda");
        assert_eq!(
            blobs.get(&digest).expect("lee").as_deref(),
            Some(&b"unos bytes cualesquiera"[..])
        );
    }

    #[test]
    fn the_same_content_is_stored_once() {
        let (_dir, blobs) = temporary();
        let first = blobs.put(b"repetido").expect("guarda");
        let second = blobs.put(b"repetido").expect("guarda otra vez");
        assert_eq!(first, second, "el nombre es el contenido");
    }

    #[test]
    fn different_content_never_shares_a_name() {
        let (_dir, blobs) = temporary();
        assert_ne!(
            blobs.put(b"uno").expect("a"),
            blobs.put(b"otro").expect("b")
        );
    }

    #[test]
    fn an_io_error_that_is_not_a_missing_file_is_not_swallowed() {
        let (_dir, blobs) = temporary();
        let digest = "a".repeat(64);
        std::fs::create_dir_all(blobs.path_for(&digest)).expect("crea carpeta");
        assert!(
            blobs.get(&digest).is_err(),
            "un error de E/S real no puede devolver Ok(None)"
        );
    }

    #[test]
    fn asking_for_something_that_is_not_there_is_not_an_error() {
        let (_dir, blobs) = temporary();
        let missing = "0".repeat(64);
        assert!(blobs.get(&missing).expect("lee").is_none());
        assert!(!blobs.exists(&missing));
        blobs
            .remove(&missing)
            .expect("borrar lo que no está no falla");
    }

    #[test]
    fn deleting_overwrites_before_unlinking() {
        let (_dir, blobs) = temporary();
        let digest = blobs.put(b"contrasena del banco").expect("guarda");
        assert!(blobs.exists(&digest));
        blobs.remove(&digest).expect("borra");
        assert!(!blobs.exists(&digest));
        assert!(blobs.get(&digest).expect("lee").is_none());
    }

    #[test]
    fn an_empty_blob_is_still_a_blob() {
        let (_dir, blobs) = temporary();
        let digest = blobs.put(b"").expect("guarda");
        assert_eq!(blobs.get(&digest).expect("lee"), Some(Vec::new()));
    }

    #[test]
    fn nothing_is_left_behind_when_writing_succeeds() {
        let (dir, blobs) = temporary();
        blobs.put(b"algo").expect("guarda");
        let partials = files_under(&dir.path().join("blobs"))
            .into_iter()
            .filter(|p| p.extension().is_some_and(|e| e == "partial"))
            .count();
        assert_eq!(partials, 0, "el temporal se renombra, no se queda");
    }

    fn aged(path: &Path) {
        let file = std::fs::File::options()
            .write(true)
            .open(path)
            .expect("abre");
        file.set_modified(std::time::UNIX_EPOCH).expect("envejece");
    }

    #[test]
    fn a_blob_nobody_references_is_removed_once_it_is_old_enough() {
        let (_dir, blobs) = temporary();
        let digest = blobs.put(b"huerfano").expect("guarda");
        assert_eq!(
            blobs.sweep(&|_| false).expect("barre"),
            0,
            "recién escrito: puede ser de un ítem a medio guardar"
        );
        aged(&blobs.path_for(&digest));
        assert_eq!(blobs.sweep(&|_| false).expect("barre"), 1);
        assert!(!blobs.exists(&digest));
    }

    #[test]
    fn a_referenced_blob_survives_the_sweep_however_old() {
        let (_dir, blobs) = temporary();
        let digest = blobs.put(b"con dueno").expect("guarda");
        aged(&blobs.path_for(&digest));
        let keep = digest.clone();
        assert_eq!(blobs.sweep(&|name| name == keep).expect("barre"), 0);
        assert!(blobs.exists(&digest));
    }

    #[test]
    fn a_partial_file_left_by_a_crash_goes_too() {
        let (_dir, blobs) = temporary();
        let digest = "b".repeat(64);
        let leftover = blobs.path_for(&digest).with_extension("partial");
        std::fs::create_dir_all(leftover.parent().expect("padre")).expect("carpeta");
        std::fs::write(&leftover, b"a medias").expect("escribe");
        aged(&leftover);
        assert_eq!(blobs.sweep(&|_| true).expect("barre"), 1);
        assert!(!leftover.exists(), "aunque su digest esté referenciado");
    }

    #[test]
    fn sweeping_an_empty_store_is_nothing() {
        let (_dir, blobs) = temporary();
        assert_eq!(blobs.sweep(&|_| false).expect("barre"), 0);
    }

    #[test]
    fn a_file_that_is_not_a_blob_is_neither_touched_nor_counted() {
        let (dir, blobs) = temporary();
        let root = dir.path().join("blobs");
        let strays = [
            root.join(".DS_Store"),
            root.join("x"),
            root.join("ñ.txt"),
            root.join("ab").join("cd").join("notes.partial"),
        ];
        for stray in &strays {
            std::fs::create_dir_all(stray.parent().expect("padre")).expect("carpeta");
            std::fs::write(stray, b"ajeno").expect("escribe");
            aged(stray);
        }
        assert_eq!(blobs.sweep(&|_| false).expect("barre"), 0);
        for stray in &strays {
            assert!(stray.exists(), "{} no era nuestro", stray.display());
        }
    }

    #[test]
    fn only_a_digest_shaped_name_is_a_blob() {
        let digest = "0123456789abcdef".repeat(4);
        assert_eq!(digest_of(&digest), Some((digest.as_str(), false)));
        let partial = format!("{digest}.partial");
        assert_eq!(digest_of(&partial), Some((digest.as_str(), true)));
        assert_eq!(digest_of(".DS_Store"), None);
        assert_eq!(digest_of(&"g".repeat(64)), None, "no es hexadecimal");
        assert_eq!(digest_of(&"a".repeat(63)), None, "le falta uno");
        assert_eq!(digest_of("x.partial"), None);
    }

    #[test]
    fn reclaiming_a_blob_that_already_exists_makes_it_fresh_again() {
        let (_dir, blobs) = temporary();
        let digest = blobs.put(b"reclamado").expect("guarda");
        aged(&blobs.path_for(&digest));
        blobs.put(b"reclamado").expect("otra vez");
        assert_eq!(
            blobs.sweep(&|_| false).expect("barre"),
            0,
            "quien lo acaba de reclamar aún no ha escrito su fila"
        );
        assert!(blobs.exists(&digest));
    }
}
