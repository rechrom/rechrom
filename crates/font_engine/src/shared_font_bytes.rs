//! Shared immutable font data, retaining either an allocation or a read-only mapping.
use std::{fmt, ops::Deref, sync::Arc};

#[derive(Clone)]
pub struct SharedFontBytes(Arc<Backing>);

enum Backing {
    Owned(Arc<[u8]>),
    #[cfg(unix)]
    Mapped(ReadOnlyMapping),
}

impl SharedFontBytes {
    /// Whether these handles retain the same underlying storage, without reading it.
    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        Arc::ptr_eq(&a.0, &b.0)
            || match (&*a.0, &*b.0) {
                (Backing::Owned(a), Backing::Owned(b)) => Arc::ptr_eq(a, b),
                #[cfg(unix)]
                _ => false,
            }
    }

    pub fn is_mapped(&self) -> bool {
        match &*self.0 {
            Backing::Owned(_) => false,
            #[cfg(unix)]
            Backing::Mapped(_) => true,
        }
    }

    /// Map the entire regular file with PROT_READ | MAP_PRIVATE. No pages are
    /// prefaulted: their first access is paid for by the actual font consumer.
    ///
    /// # Safety
    /// The caller must ensure the file's content and length cannot change for
    /// the lifetime of this owner and every clone, including retained native
    /// font/blob handles. A read-only mapping does not protect against another
    /// process modifying or truncating the underlying file. Closing the file
    /// or unlinking it does not invalidate the mapping.
    #[cfg(unix)]
    pub unsafe fn map_readonly(file: &std::fs::File) -> std::io::Result<Self> {
        use std::{io, os::fd::AsRawFd, ptr::NonNull};
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "font mapping requires a regular file",
            ));
        }
        let length = usize::try_from(metadata.len())
            .ok()
            .filter(|&n| n <= isize::MAX as usize)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "font size exceeds address space",
                )
            })?;
        if length == 0 {
            return Ok(Self::default());
        }
        // SAFETY: The descriptor is live and length was checked. The caller's
        // contract guarantees immutable file storage for all returned owners.
        let raw = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                length,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };
        if raw == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        // Rust slices require a non-null pointer even if the OS could map zero.
        let Some(pointer) = NonNull::new(raw.cast::<u8>()) else {
            unsafe { libc::munmap(raw, length) };
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "font mapping returned a null address",
            ));
        };
        Ok(Self(Arc::new(Backing::Mapped(ReadOnlyMapping {
            pointer,
            length,
            #[cfg(test)]
            drop_observer: None,
        }))))
    }

    #[cfg(all(test, unix))]
    pub(crate) fn observe_mapping_drop(&mut self, observer: Arc<std::sync::atomic::AtomicUsize>) {
        let Backing::Mapped(mapping) = Arc::get_mut(&mut self.0).expect("unique mapping owner")
        else {
            panic!("expected a mapped font");
        };
        mapping.drop_observer = Some(observer);
    }
}

impl Deref for SharedFontBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match &*self.0 {
            Backing::Owned(bytes) => bytes,
            #[cfg(unix)]
            Backing::Mapped(mapping) => {
                // SAFETY: The entire nonempty file is mapped, the pointer and
                // length are valid for a slice, and self retains its owner.
                // Immutable file storage is required by map_readonly's caller.
                unsafe { std::slice::from_raw_parts(mapping.pointer.as_ptr(), mapping.length) }
            }
        }
    }
}

impl AsRef<[u8]> for SharedFontBytes {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl PartialEq for SharedFontBytes {
    fn eq(&self, other: &Self) -> bool {
        Self::ptr_eq(self, other) || **self == **other
    }
}
impl Eq for SharedFontBytes {}

impl fmt::Debug for SharedFontBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedFontBytes")
            .field("len", &self.len())
            .field("mapped", &self.is_mapped())
            .finish()
    }
}

impl Default for SharedFontBytes {
    fn default() -> Self {
        Arc::<[u8]>::default().into()
    }
}
impl From<Arc<[u8]>> for SharedFontBytes {
    fn from(bytes: Arc<[u8]>) -> Self {
        Self(Arc::new(Backing::Owned(bytes)))
    }
}
impl From<Vec<u8>> for SharedFontBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Arc::<[u8]>::from(bytes).into()
    }
}
impl From<&[u8]> for SharedFontBytes {
    fn from(bytes: &[u8]) -> Self {
        Arc::<[u8]>::from(bytes).into()
    }
}
impl<const N: usize> From<&[u8; N]> for SharedFontBytes {
    fn from(bytes: &[u8; N]) -> Self {
        bytes.as_slice().into()
    }
}

#[cfg(unix)]
struct ReadOnlyMapping {
    pointer: std::ptr::NonNull<u8>,
    length: usize,
    #[cfg(test)]
    drop_observer: Option<Arc<std::sync::atomic::AtomicUsize>>,
}
// SAFETY: Only immutable slices are exposed. The unsafe mapping constructor
// requires immutable file storage throughout all owners, across all threads.
// Arc keeps the mapping alive until every reader has released its owner.
#[cfg(unix)]
unsafe impl Send for ReadOnlyMapping {}
#[cfg(unix)]
unsafe impl Sync for ReadOnlyMapping {}

#[cfg(unix)]
impl Drop for ReadOnlyMapping {
    fn drop(&mut self) {
        // SAFETY: This is the unique mapping owner, dropped once by the last
        // Arc. No slice or native font pointer can outlive its retained owner.
        unsafe { libc::munmap(self.pointer.as_ptr().cast(), self.length) };
        #[cfg(test)]
        if let Some(observer) = &self.drop_observer {
            observer.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_storage_shares_without_copying_and_compares_content() {
        let bytes: Arc<[u8]> = b"font bytes".as_slice().into();
        let a = SharedFontBytes::from(bytes.clone());
        let b = SharedFontBytes::from(bytes);
        assert!(SharedFontBytes::ptr_eq(&a, &b));
        assert_eq!(a.as_ptr(), b.as_ptr());
        assert!(a == SharedFontBytes::from(b"font bytes"));
        assert!(a != SharedFontBytes::from(b"other font"));
        assert!(!a.is_mapped());
        assert!(SharedFontBytes::default().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn mapping_survives_file_and_owner_drop_until_last_cross_thread_clone() {
        use std::{
            io::Write,
            sync::atomic::{AtomicUsize, Ordering},
        };
        let path =
            std::env::temp_dir().join(format!("font-mapped-owner-{}.bin", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        std::fs::remove_file(&path).unwrap();
        file.write_all(b"mapped immutable font bytes").unwrap();
        // SAFETY: The only writable descriptor is dropped immediately below;
        // the unlinked file cannot be accessed or mutated by other handles.
        let mut bytes = unsafe { SharedFontBytes::map_readonly(&file) }.unwrap();
        drop(file);
        let observer = Arc::new(AtomicUsize::new(0));
        bytes.observe_mapping_drop(observer.clone());
        let clone = bytes.clone();
        assert!(SharedFontBytes::ptr_eq(&bytes, &clone));
        assert!(bytes.is_mapped());
        let pointer = bytes.as_ptr() as usize;
        drop(bytes);
        assert_eq!(observer.load(Ordering::SeqCst), 0);
        std::thread::spawn(move || {
            assert_eq!(clone.as_ptr() as usize, pointer);
            assert_eq!(clone.as_ref(), b"mapped immutable font bytes");
        })
        .join()
        .unwrap();
        assert_eq!(observer.load(Ordering::SeqCst), 1);
    }

    #[cfg(unix)]
    #[test]
    fn mapping_empty_and_invalid_files_preserves_errors() {
        use std::io::Write;
        let path = std::env::temp_dir().join(format!("font-empty-map-{}.bin", std::process::id()));
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        std::fs::remove_file(path).unwrap();
        // SAFETY: This unlinked file remains empty and is never written.
        let empty = unsafe { SharedFontBytes::map_readonly(&file) }.unwrap();
        assert!(empty.is_empty());
        assert!(!empty.is_mapped());
        let device = std::fs::File::open("/dev/null").unwrap();
        assert_eq!(
            unsafe { SharedFontBytes::map_readonly(&device) }
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
        let write_only = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/null")
            .unwrap();
        assert!(unsafe { SharedFontBytes::map_readonly(&write_only) }.is_err());
        let path =
            std::env::temp_dir().join(format!("font-write-only-map-{}.bin", std::process::id()));
        let mut write_only = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        std::fs::remove_file(path).unwrap();
        write_only.write_all(b"font").unwrap();
        // SAFETY: The unlinked regular file is never written after this call.
        // mmap must report the unreadable descriptor rather than expose bytes.
        assert!(unsafe { SharedFontBytes::map_readonly(&write_only) }.is_err());
    }
}
