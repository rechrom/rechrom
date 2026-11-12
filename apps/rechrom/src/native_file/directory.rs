//! Browser-process directory enumeration. The granted root descriptor is the
//! authority, and every child is opened relative to an admitted directory.
//! Symlinks/reparse points never widen the native drop's filesystem grant.
use super::{metadata_for_file, OfferedReader};
use fileapi::{File, NativeFileError};
use std::{path::Path, sync::Arc};

fn io(error: std::io::Error) -> NativeFileError {
    NativeFileError::Io(error.to_string())
}

pub(super) fn enumerate(
    root: Arc<std::fs::File>,
    selected: &File,
    executor: Arc<dyn executor::Executor>,
    access: Option<Arc<dyn winit::event::NativeDragFileAccess>>,
) -> Result<Vec<Arc<File>>, NativeFileError> {
    if !root.metadata().map_err(io)?.is_dir() {
        return Err(NativeFileError::Io(
            "dropped item is not a directory".into(),
        ));
    }
    let mut files = Vec::new();
    walk(
        &root,
        Path::new(selected.GetPath()),
        selected.name(),
        &executor,
        &access,
        &mut files,
    )?;
    Ok(files)
}
fn walk(
    directory: &std::fs::File,
    display_path: &Path,
    relative: &str,
    executor: &Arc<dyn executor::Executor>,
    access: &Option<Arc<dyn winit::event::NativeDragFileAccess>>,
    output: &mut Vec<Arc<File>>,
) -> Result<(), NativeFileError> {
    let mut children = platform::children(directory).map_err(io)?;
    // Stable chooser ordering; no script-visible absolute path is reparsed.
    children.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, handle) in children {
        let metadata = handle.metadata().map_err(io)?;
        if platform::is_link(&metadata) {
            continue;
        }
        let path = display_path.join(&name);
        let path_string = path
            .to_str()
            .ok_or_else(|| NativeFileError::Io("directory entry path is not UTF-8".into()))?;
        let relative = format!("{relative}/{name}");
        if metadata.is_dir() {
            walk(&handle, &path, &relative, executor, access, output)?;
        } else if metadata.is_file() {
            let content_type = match crate::native_mime::from_file(path_string) {
                Ok(value) => value.unwrap_or_default(),
                Err(NativeFileError::Unavailable) => String::new(),
                Err(error) => return Err(error),
            };
            let snapshot = metadata_for_file(&metadata, content_type.clone())?;
            let reader = Arc::new(OfferedReader {
                file: Arc::new(handle),
                access: access.clone(),
                content_type: content_type.clone(),
                executor: executor.clone(),
            });
            output.push(File::CreateForNativePath(
                path_string.into(),
                name,
                relative,
                content_type,
                Some(snapshot),
                reader,
            ));
        }
    }
    Ok(())
}

#[cfg(unix)]
mod platform {
    use std::{
        ffi::{CStr, CString},
        fs::File,
        os::fd::{AsRawFd, FromRawFd},
    };
    pub(super) fn is_link(metadata: &std::fs::Metadata) -> bool {
        metadata.file_type().is_symlink()
    }
    pub(super) fn children(parent: &File) -> std::io::Result<Vec<(String, File)>> {
        // Opening "." against the live descriptor gives each enumeration its
        // own directory offset, without resolving the original offered path.
        let fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                c".".as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let stream = unsafe { libc::fdopendir(fd) };
        if stream.is_null() {
            unsafe {
                libc::close(fd);
            }
            return Err(std::io::Error::last_os_error());
        }
        struct Stream(*mut libc::DIR);
        impl Drop for Stream {
            fn drop(&mut self) {
                unsafe {
                    libc::closedir(self.0);
                }
            }
        }
        let stream = Stream(stream);
        let mut result = Vec::new();
        loop {
            // readdir signals both EOF and errors with null; clear errno so a
            // partial traversal can never be published as a successful chooser.
            #[cfg(target_os = "macos")]
            unsafe {
                *libc::__error() = 0;
            }
            #[cfg(target_os = "linux")]
            unsafe {
                *libc::__errno_location() = 0;
            }
            let entry = unsafe { libc::readdir(stream.0) };
            if entry.is_null() {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error().unwrap_or(0) != 0 {
                    return Err(error);
                }
                break;
            }
            let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
            if name.to_bytes() == b"." || name.to_bytes() == b".." {
                continue;
            }
            let name = CString::new(name.to_bytes())?;
            let child = unsafe {
                libc::openat(
                    fd,
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                )
            };
            if child < 0 {
                let error = std::io::Error::last_os_error();
                // Chromium's recursive file chooser excludes symbolic links.
                if error.raw_os_error() == Some(libc::ELOOP) {
                    continue;
                }
                return Err(error);
            }
            let child = unsafe { File::from_raw_fd(child) };
            let name = name
                .to_str()
                .map_err(|_| std::io::Error::other("directory entry name is not UTF-8"))?
                .to_owned();
            result.push((name, child));
        }
        Ok(result)
    }
}

#[cfg(windows)]
mod platform {
    use std::{
        ffi::c_void,
        fs::File,
        os::windows::{
            fs::MetadataExt,
            io::{AsRawHandle, FromRawHandle},
        },
    };
    type Handle = *mut c_void;
    #[repr(C)]
    struct UnicodeString {
        length: u16,
        maximum_length: u16,
        buffer: *mut u16,
    }
    #[repr(C)]
    struct ObjectAttributes {
        length: u32,
        root: Handle,
        name: *mut UnicodeString,
        attributes: u32,
        security: *mut c_void,
        quality: *mut c_void,
    }
    #[repr(C)]
    struct IoStatus {
        status: usize,
        information: usize,
    }
    #[link(name = "ntdll")]
    extern "system" {
        fn NtCreateFile(
            handle: *mut Handle,
            access: u32,
            attributes: *mut ObjectAttributes,
            status: *mut IoStatus,
            allocation: *const i64,
            file_attributes: u32,
            share: u32,
            disposition: u32,
            options: u32,
            ea: *const c_void,
            ea_length: u32,
        ) -> i32;
        fn NtQueryDirectoryFile(
            handle: Handle,
            event: Handle,
            routine: *mut c_void,
            context: *mut c_void,
            status: *mut IoStatus,
            buffer: *mut c_void,
            length: u32,
            class: u32,
            single: u8,
            name: *const UnicodeString,
            restart: u8,
        ) -> i32;
        fn RtlNtStatusToDosError(status: i32) -> u32;
    }
    fn error(status: i32) -> std::io::Error {
        std::io::Error::from_raw_os_error(unsafe { RtlNtStatusToDosError(status) } as i32)
    }
    fn open(parent: &File, name: &str) -> std::io::Result<File> {
        let mut name: Vec<u16> = name.encode_utf16().collect();
        let length = u16::try_from(name.len() * 2)
            .map_err(|_| std::io::Error::other("directory entry name too long"))?;
        let mut name = UnicodeString {
            length,
            maximum_length: length,
            buffer: name.as_mut_ptr(),
        };
        let mut attributes = ObjectAttributes {
            length: std::mem::size_of::<ObjectAttributes>() as u32,
            root: parent.as_raw_handle(),
            name: &mut name,
            attributes: 0x40,
            security: std::ptr::null_mut(),
            quality: std::ptr::null_mut(),
        };
        let mut status = IoStatus {
            status: 0,
            information: 0,
        };
        let mut handle = std::ptr::null_mut();
        // FILE_OPEN_REPARSE_POINT prevents following junctions/symlinks;
        // metadata below rejects every reparse point before recursion/read.
        let result = unsafe {
            NtCreateFile(
                &mut handle,
                0x00100081,
                &mut attributes,
                &mut status,
                std::ptr::null(),
                0,
                7,
                1,
                0x00204020,
                std::ptr::null(),
                0,
            )
        };
        if result < 0 {
            return Err(error(result));
        }
        Ok(unsafe { File::from_raw_handle(handle) })
    }
    pub(super) fn is_link(metadata: &std::fs::Metadata) -> bool {
        metadata.file_attributes() & 0x400 != 0
    }
    pub(super) fn children(parent: &File) -> std::io::Result<Vec<(String, File)>> {
        let directory = open(parent, ".")?;
        let mut buffer = vec![0u64; 8192];
        let mut first = true;
        let mut result = Vec::new();
        loop {
            let mut status = IoStatus {
                status: 0,
                information: 0,
            };
            let returned = unsafe {
                NtQueryDirectoryFile(
                    directory.as_raw_handle(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut status,
                    buffer.as_mut_ptr().cast(),
                    (buffer.len() * 8) as u32,
                    12,
                    0,
                    std::ptr::null(),
                    u8::from(first),
                )
            };
            first = false;
            if returned as u32 == 0x80000006 {
                break;
            } // STATUS_NO_MORE_FILES
            if returned < 0 {
                return Err(error(returned));
            }
            let bytes = unsafe {
                std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), status.information)
            };
            let mut offset = 0;
            loop {
                if offset + 12 > bytes.len() {
                    return Err(std::io::Error::other("invalid native directory record"));
                }
                let next =
                    u32::from_ne_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                let length =
                    u32::from_ne_bytes(bytes[offset + 8..offset + 12].try_into().unwrap()) as usize;
                if length % 2 != 0 || offset + 12 + length > bytes.len() {
                    return Err(std::io::Error::other("invalid native directory filename"));
                }
                let wide: Vec<u16> = bytes[offset + 12..offset + 12 + length]
                    .chunks_exact(2)
                    .map(|v| u16::from_ne_bytes([v[0], v[1]]))
                    .collect();
                let name = String::from_utf16(&wide)
                    .map_err(|_| std::io::Error::other("directory entry name is not Unicode"))?;
                if name != "." && name != ".." {
                    result.push((name.clone(), open(&directory, &name)?));
                }
                if next == 0 {
                    break;
                }
                if next < 12 || offset + next >= bytes.len() {
                    return Err(std::io::Error::other("invalid native directory offset"));
                }
                offset += next;
            }
        }
        Ok(result)
    }
}

#[cfg(not(any(unix, windows)))]
mod platform {
    pub(super) fn is_link(_: &std::fs::Metadata) -> bool {
        false
    }
    pub(super) fn children(_: &std::fs::File) -> std::io::Result<Vec<(String, std::fs::File)>> {
        Err(std::io::Error::other(
            "native directory enumeration unsupported",
        ))
    }
}
