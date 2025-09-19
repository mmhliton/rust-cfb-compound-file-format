#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::CompoundFile;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::os::raw::c_char;
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
/// - The `path` pointer must be a valid, null-terminated C string.
/// - The `callback` function pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn cfb_list_entries(
    comp: *mut CfbCompoundFile,
    path: *const c_char,
    callback: extern "C" fn(*const c_char),
) {
    if comp.is_null() || path.is_null() {
        return;
    }
    let comp = &*(comp as *mut CompoundFile<File>);
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    if let Ok(entries) = comp.read_storage(Path::new(path_str)) {
        for entry in entries {
            if let Ok(name) = CString::new(entry.name()) {
                callback(name.as_ptr());
            }
        }
    }
}
