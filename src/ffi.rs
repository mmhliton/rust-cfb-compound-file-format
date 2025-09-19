#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::CompoundFile;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::os::raw::{c_char, c_int};
use std::path::Path;

/// An opaque handle to a CompoundFile, used in the C FFI.
#[repr(C)]
pub struct CfbCompoundFile {
    _private: (),
}

/// Opens a compound file from a given path and returns an opaque pointer to it.
///
/// Returns a null pointer if the file cannot be opened.
///
/// # Safety
/// The `path` pointer must be a valid, null-terminated C string.
/// The caller is responsible for calling `cfb_close` on the returned pointer.
#[no_mangle]
pub unsafe extern "C" fn cfb_open(path: *const c_char) -> *mut CfbCompoundFile {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let file = match File::open(Path::new(path_str)) {
        Ok(f) => f,
        Err(_) => return std::ptr::null_mut(),
    };
    match CompoundFile::open(file) {
        Ok(comp) => Box::into_raw(Box::new(comp)) as *mut CfbCompoundFile,
        Err(_) => std::ptr::null_mut(),
    }
}

/// Closes a compound file and releases its resources.
///
/// # Safety
/// The `comp` pointer must be a valid pointer returned by `cfb_open`.
/// After calling this, the pointer is no longer valid and must not be used.
#[no_mangle]
pub unsafe extern "C" fn cfb_close(comp: *mut CfbCompoundFile) {
    if !comp.is_null() {
        let _ = Box::from_raw(comp as *mut CompoundFile<File>);
    }
}

/// Lists the entries within a given storage of a compound file.
///
/// # Safety
/// - The `comp` pointer must be a valid pointer returned by `cfb_open`.
/// - The `callback` function pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn cfb_list_entries(
    comp: *mut CfbCompoundFile,
    callback: extern "C" fn(*const c_char, c_int, usize, *mut std::ffi::c_void),
    user_data: *mut std::ffi::c_void,
) -> c_int {
    if comp.is_null() {
        return -1;
    }
    let comp = &*(comp as *mut CompoundFile<File>);
    
    // Walk through all entries
    let entries = comp.walk();
    
    for entry in entries {
        let path_string = entry.path().to_string_lossy();
        if let Ok(name) = CString::new(path_string.as_ref()) {
            let is_stream = if entry.is_stream() { 1 } else { 0 };
            let size = if entry.is_stream() { entry.len() as usize } else { 0 };
            callback(name.as_ptr(), is_stream, size, user_data);
        }
    }
    0
}

/// Reads stream data from a compound file.
///
/// # Safety
/// - The `comp` pointer must be a valid pointer returned by `cfb_open`.
/// - The `path` pointer must be a valid, null-terminated C string.
/// - If `buffer` is not null, it must point to at least `*size` bytes of writable memory.
/// - The `size` pointer must be valid and point to a writable usize.
#[no_mangle]
pub unsafe extern "C" fn cfb_read_stream(
    comp: *mut CfbCompoundFile,
    path: *const c_char,
    buffer: *mut u8,
    size: *mut usize,
) -> c_int {
    if comp.is_null() || path.is_null() || size.is_null() {
        return -1;
    }
    
    let comp = &mut *(comp as *mut CompoundFile<File>);
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let mut stream = match comp.open_stream(Path::new(path_str)) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let stream_size = stream.len() as usize;
    
    if buffer.is_null() {
        // Just return the size
        *size = stream_size;
        return 0;
    }
    
    if *size < stream_size {
        return -1; // Buffer too small
    }
    
    let mut data = vec![0u8; stream_size];
    if stream.read_exact(&mut data).is_err() {
        return -1;
    }
    
    std::ptr::copy_nonoverlapping(data.as_ptr(), buffer, stream_size);
    *size = stream_size;
    0
}
