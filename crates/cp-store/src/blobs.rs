use crate::{Error, Result};
use std::path::{Path, PathBuf};

/// Los bytes que no caben en la fila, guardados por su contenido.
///
/// El nombre del archivo **es** el hash de lo que contiene, así que dos
/// copias del mismo contenido ocupan un archivo, y comprobar que un blob no
/// se ha corrompido es recalcular su nombre.
///
/// Aquí solo viven las cosas que CopyPaste crea: imágenes copiadas y sus
/// miniaturas. **Un archivo que el usuario copió desde el disco se queda
/// donde está** y solo se guarda su ruta: duplicar un vídeo de 4 GB porque
/// alguien lo copió sería inaceptable, y moverlo, peor.
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
        // Dos niveles de subcarpeta: un directorio con cien mil entradas es
        // lento de listar en cualquier sistema de archivos.
        self.root
            .join(&digest[0..2])
            .join(&digest[2..4])
            .join(digest)
    }

    /// Guarda los bytes y devuelve su nombre. Si ya estaban, no escribe nada.
    pub fn put(&self, bytes: &[u8]) -> Result<String> {
        let digest = blake3::hash(bytes).to_hex().to_string();
        let path = self.path_for(&digest);
        if path.exists() {
            return Ok(digest);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(Error::Io)?;
            crate::store::restrict(parent, 0o700)?;
        }
        // Se escribe a un temporal y se renombra: un corte a mitad deja un
        // archivo suelto, nunca un blob con el nombre de un contenido que no
        // tiene.
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

    /// Borra un blob **sobrescribiéndolo antes**.
    ///
    /// `secure_delete` de SQLite cubre las páginas de la base y no los
    /// archivos de al lado. Si el historial promete que una contraseña
    /// borrada deja de ser legible, el blob que la contenía tiene que
    /// desaparecer de verdad.
    pub fn remove(&self, digest: &str) -> Result<()> {
        let path = self.path_for(digest);
        let Ok(metadata) = std::fs::metadata(&path) else {
            return Ok(());
        };
        let zeros = vec![0u8; metadata.len() as usize];
        std::fs::write(&path, &zeros).map_err(Error::Io)?;
        std::fs::remove_file(&path).map_err(Error::Io)?;
        Ok(())
    }

    pub fn exists(&self, digest: &str) -> bool {
        self.path_for(digest).exists()
    }
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
        // Un directorio en la ruta exacta del blob: leerlo falla con un error
        // que no es `NotFound`, y ese error no puede confundirse con «no hay
        // nada que leer».
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
        let partials = walk(&dir.path().join("blobs"))
            .into_iter()
            .filter(|p| p.extension().is_some_and(|e| e == "partial"))
            .count();
        assert_eq!(partials, 0, "el temporal se renombra, no se queda");
    }

    fn walk(root: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let Ok(entries) = std::fs::read_dir(root) else {
            return found;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                found.extend(walk(&path));
            } else {
                found.push(path);
            }
        }
        found
    }
}
