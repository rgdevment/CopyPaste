use crate::{Error, Result};
use std::path::{Path, PathBuf};

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

    pub fn remove_if_settled(&self, digest: &str) -> Result<bool> {
        let path = self.path_for(digest);
        if !path.exists() || is_fresh(&path) {
            return Ok(false);
        }
        remove_at(&path).map(|()| true)
    }

    pub fn exists(&self, digest: &str) -> bool {
        self.path_for(digest).exists()
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
    let shaped = digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit());
    shaped.then_some((digest, partial))
}

fn remove_at(path: &Path) -> Result<()> {
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
    let zeros = vec![0u8; metadata.len() as usize];
    file.write_all(&zeros).map_err(Error::Io)?;
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
        let (dir, blobs) = temporary();
        let digest = blobs.put(b"contrasena del banco").expect("guarda");
        let twin = dir.path().join("gemelo");
        std::fs::hard_link(blobs.path_for(&digest), &twin).expect("enlace duro");
        blobs.remove(&digest).expect("borra");
        assert!(!blobs.exists(&digest));
        assert!(blobs.get(&digest).expect("lee").is_none());
        let bytes = std::fs::read(&twin).expect("el otro nombre sigue");
        assert_eq!(bytes.len(), b"contrasena del banco".len());
        assert!(
            bytes.iter().all(|b| *b == 0),
            "los bloques se pisaron con ceros antes de soltar el nombre"
        );
    }

    #[test]
    fn a_symlink_named_like_a_digest_is_unlinked_never_followed() {
        let (dir, blobs) = temporary();
        let victim = dir.path().join("ajeno.txt");
        std::fs::write(&victim, b"no me toques").expect("archivo");
        let digest = "c".repeat(64);
        let link = blobs.path_for(&digest);
        std::fs::create_dir_all(link.parent().expect("padre")).expect("carpeta");
        std::os::unix::fs::symlink(&victim, &link).expect("enlace");
        aged(&victim);
        assert_eq!(blobs.sweep(&|_| false).expect("barre"), 1);
        assert!(!link.exists() && std::fs::symlink_metadata(&link).is_err());
        assert_eq!(
            std::fs::read(&victim).expect("sigue"),
            b"no me toques",
            "se borra el enlace, nunca lo que hay detrás"
        );
    }

    #[test]
    fn a_directory_named_like_a_digest_is_left_alone() {
        let (_dir, blobs) = temporary();
        let digest = "d".repeat(64);
        let folder = blobs.path_for(&digest);
        std::fs::create_dir_all(&folder).expect("carpeta");
        assert_eq!(blobs.sweep(&|_| false).expect("barre"), 0);
        assert!(folder.is_dir());
        blobs
            .remove(&digest)
            .expect("borrar una carpeta no es borrar un blob");
        assert!(folder.is_dir());
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
    fn a_fresh_blob_is_not_removed_by_a_release_only_by_the_sweep_later() {
        let (_dir, blobs) = temporary();
        let digest = blobs.put(b"recien escrito").expect("guarda");
        assert!(
            !blobs.remove_if_settled(&digest).expect("no toca"),
            "otra conexión puede estar a punto de referenciarlo"
        );
        assert!(blobs.exists(&digest));
        aged(&blobs.path_for(&digest));
        assert!(blobs.remove_if_settled(&digest).expect("ahora sí"));
        assert!(!blobs.exists(&digest));
        assert!(
            !blobs.remove_if_settled(&digest).expect("ya no está"),
            "borrar dos veces no es error"
        );
    }

    #[test]
    fn two_writers_never_share_a_temporary_file() {
        let (dir, blobs) = temporary();
        let a = blobs.put(b"uno").expect("a");
        let b = blobs.put(b"dos").expect("b");
        assert_ne!(a, b);
        let leftovers = files_under(&dir.path().join("blobs"))
            .into_iter()
            .filter(|p| p.to_string_lossy().contains(".partial"))
            .count();
        assert_eq!(leftovers, 0);
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
        let private = format!("{digest}.4242-7.partial");
        assert_eq!(digest_of(&private), Some((digest.as_str(), true)));
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
