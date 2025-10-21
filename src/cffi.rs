//! C Foreign Function Interface (FFI) for the CFB library
//! 
//! This module provides C-compatible bindings for the Rust CFB library,
//! enabling integration with C and C++ applications.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::{CompoundFile, Version};
use std::ffi::{CStr, CString};
use std::io::{Cursor, Read, Write};
use std::os::raw::{c_char, c_int};
use std::path::Path;

/// An opaque handle to a CompoundFile, used in the C FFI.
#[repr(C)]
pub struct CfbCompoundFile {
    _private: (),
}

/// An opaque handle to a CompoundFile using in-memory storage, used in the C FFI.
#[repr(C)]
pub struct CfbMemoryCompoundFile {
    _private: (),
}

//===========================================================================//
// File-based compound file operations
//===========================================================================//

// Note: cfb_open and cfb_close are in ffi.rs for original file-based operations

// Note: cfb_list_entries is in ffi.rs for original file-based operations

// Note: cfb_read_stream is in ffi.rs for original file-based operations

//===========================================================================//
// Memory-based compound file operations
//===========================================================================//

/// Creates a new compound file in memory.
///
/// Returns a null pointer if creation fails.
#[no_mangle]
pub unsafe extern "C" fn cfb_create_memory() -> *mut CfbMemoryCompoundFile {
    let cursor = Cursor::new(Vec::new());
    match CompoundFile::create(cursor) {
        Ok(comp) => Box::into_raw(Box::new(comp)) as *mut CfbMemoryCompoundFile,
        Err(_) => std::ptr::null_mut(),
    }
}

/// Creates a new compound file in memory with a specific version.
///
/// version: 3 for V3, 4 for V4
/// Returns a null pointer if creation fails.
#[no_mangle]
pub unsafe extern "C" fn cfb_create_memory_with_version(version: c_int) -> *mut CfbMemoryCompoundFile {
    let ver = match version {
        3 => Version::V3,
        4 => Version::V4,
        _ => return std::ptr::null_mut(),
    };
    let cursor = Cursor::new(Vec::new());
    match CompoundFile::create_with_version(ver, cursor) {
        Ok(comp) => Box::into_raw(Box::new(comp)) as *mut CfbMemoryCompoundFile,
        Err(_) => std::ptr::null_mut(),
    }
}

/// Closes a memory-based compound file and releases its resources.
///
/// # Safety
/// The `comp` pointer must be a valid pointer returned by `cfb_create_memory`.
/// After calling this, the pointer is no longer valid and must not be used.
#[no_mangle]
pub unsafe extern "C" fn cfb_close_memory(comp: *mut CfbMemoryCompoundFile) {
    if !comp.is_null() {
        let _ = Box::from_raw(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    }
}

//===========================================================================//
// Storage operations
//===========================================================================//

/// Creates a new storage in the compound file.
///
/// Returns 0 on success, -1 on failure.
#[no_mangle]
pub unsafe extern "C" fn cfb_create_storage(
    comp: *mut CfbMemoryCompoundFile,
    path: *const c_char,
) -> c_int {
    if comp.is_null() || path.is_null() {
        return -1;
    }
    
    let comp = &mut *(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    match comp.create_storage(Path::new(path_str)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Creates a new storage in the compound file, creating parent directories as needed.
///
/// Returns 0 on success, -1 on failure.
#[no_mangle]
pub unsafe extern "C" fn cfb_create_storage_all(
    comp: *mut CfbMemoryCompoundFile,
    path: *const c_char,
) -> c_int {
    if comp.is_null() || path.is_null() {
        return -1;
    }
    
    let comp = &mut *(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    match comp.create_storage_all(Path::new(path_str)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

//===========================================================================//
// Stream operations
//===========================================================================//

/// Creates a new stream in the compound file.
///
/// Returns 0 on success, -1 on failure.
#[no_mangle]
pub unsafe extern "C" fn cfb_create_stream(
    comp: *mut CfbMemoryCompoundFile,
    path: *const c_char,
) -> c_int {
    if comp.is_null() || path.is_null() {
        return -1;
    }
    
    let comp = &mut *(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    match comp.create_stream(Path::new(path_str)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Writes data to a stream in the compound file.
///
/// Returns 0 on success, -1 on failure.
#[no_mangle]
pub unsafe extern "C" fn cfb_write_stream(
    comp: *mut CfbMemoryCompoundFile,
    path: *const c_char,
    data: *const u8,
    size: usize,
) -> c_int {
    if comp.is_null() || path.is_null() || data.is_null() {
        return -1;
    }
    
    let comp = &mut *(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let data_slice = std::slice::from_raw_parts(data, size);
    
    match comp.open_stream(Path::new(path_str)) {
        Ok(mut stream) => {
            match stream.write_all(data_slice) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        },
        Err(_) => -1,
    }
}

/// Reads stream data from a memory-based compound file.
///
/// # Safety
/// - The `comp` pointer must be a valid pointer returned by `cfb_create_memory`.
/// - The `path` pointer must be a valid, null-terminated C string.
/// - If `buffer` is not null, it must point to at least `*size` bytes of writable memory.
/// - The `size` pointer must be valid and point to a writable usize.
#[no_mangle]
pub unsafe extern "C" fn cfb_read_stream_memory(
    comp: *mut CfbMemoryCompoundFile,
    path: *const c_char,
    buffer: *mut u8,
    size: *mut usize,
) -> c_int {
    if comp.is_null() || path.is_null() || size.is_null() {
        return -1;
    }
    
    let comp = &mut *(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
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

/// Sets the length of a stream in the compound file.
///
/// Returns 0 on success, -1 on failure.
#[no_mangle]
pub unsafe extern "C" fn cfb_set_stream_len(
    comp: *mut CfbMemoryCompoundFile,
    path: *const c_char,
    new_len: usize,
) -> c_int {
    if comp.is_null() || path.is_null() {
        return -1;
    }
    
    let comp = &mut *(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    match comp.open_stream(Path::new(path_str)) {
        Ok(mut stream) => {
            match stream.set_len(new_len as u64) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        },
        Err(_) => -1,
    }
}

//===========================================================================//
// Query operations
//===========================================================================//

/// Checks if a path exists in the compound file.
///
/// Returns 1 if exists, 0 if not, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn cfb_exists(
    comp: *mut CfbMemoryCompoundFile,
    path: *const c_char,
) -> c_int {
    if comp.is_null() || path.is_null() {
        return -1;
    }
    
    let comp = &*(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    if comp.exists(Path::new(path_str)) { 1 } else { 0 }
}

/// Checks if a path is a stream in the compound file.
///
/// Returns 1 if is stream, 0 if not, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn cfb_is_stream(
    comp: *mut CfbMemoryCompoundFile,
    path: *const c_char,
) -> c_int {
    if comp.is_null() || path.is_null() {
        return -1;
    }
    
    let comp = &*(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    if comp.is_stream(Path::new(path_str)) { 1 } else { 0 }
}

/// Checks if a path is a storage in the compound file.
///
/// Returns 1 if is storage, 0 if not, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn cfb_is_storage(
    comp: *mut CfbMemoryCompoundFile,
    path: *const c_char,
) -> c_int {
    if comp.is_null() || path.is_null() {
        return -1;
    }
    
    let comp = &*(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    let c_str = CStr::from_ptr(path);
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    if comp.is_storage(Path::new(path_str)) { 1 } else { 0 }
}

/// Gets the version of the compound file.
///
/// Returns 3 for V3, 4 for V4, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn cfb_version(comp: *mut CfbMemoryCompoundFile) -> c_int {
    if comp.is_null() {
        return -1;
    }
    
    let comp = &*(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    match comp.version() {
        Version::V3 => 3,
        Version::V4 => 4,
    }
}

//===========================================================================//
// Entry listing operations
//===========================================================================//

/// Lists entries in a storage of a memory-based compound file.
///
/// # Safety
/// - The `comp` pointer must be a valid pointer returned by `cfb_create_memory`.
/// - The `path` pointer must be a valid, null-terminated C string (can be null for root).
/// - The `callback` function pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn cfb_list_entries_memory(
    comp: *mut CfbMemoryCompoundFile,
    path: *const c_char,
    callback: extern "C" fn(*const c_char, c_int, usize, *mut std::ffi::c_void),
    user_data: *mut std::ffi::c_void,
) -> c_int {
    if comp.is_null() {
        return -1;
    }
    let comp = &*(comp as *mut CompoundFile<Cursor<Vec<u8>>>);
    
    let entries = if path.is_null() {
        comp.walk()
    } else {
        let c_str = CStr::from_ptr(path);
        let path_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        match comp.walk_storage(Path::new(path_str)) {
            Ok(iter) => iter,
            Err(_) => return -1,
        }
    };
    
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