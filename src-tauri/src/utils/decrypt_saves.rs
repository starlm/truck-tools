// Silence the compiler when building for Linux
#![cfg_attr(target_os = "linux", allow(dead_code, unused_imports))]

use libloading::{Library, Symbol};
use std::fs::{File, write};
use std::io::Read;
use std::os::raw::c_uint;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

pub static USE_DECRYPT_TRUCK: LazyLock<AtomicBool> = LazyLock::new(|| AtomicBool::new(false));

const DLL_DIR: &str = "resources/SII_Decrypt.dll";

type GetMemoryFormatType = extern "C" fn(arr_val: *const u8, leng: c_uint) -> c_uint;
type DecodeMemoryType = extern "C" fn(
    arr_val: *const u8,
    leng: c_uint,
    out_buf_ptr: *const u8,
    out_buf_ptr_leng: *const c_uint,
) -> c_uint;

fn read_file_bin(path: &str) -> Option<Vec<u8>> {
    let mut file = match File::open(path) {
        Ok(res) => res,
        Err(_) => return None,
    };

    let mut buffer = Vec::new();
    match file.read_to_end(&mut buffer) {
        Ok(_) => Some(buffer),
        Err(_) => None,
    }
}

fn save_file(path: &String, file_data: &String) -> bool {
    match write(path, file_data) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(target_os = "windows")]
fn get_memory_format(bin_file: &Vec<u8>) -> Option<u32> {
    let lib = match unsafe { Library::new(format!("{}", DLL_DIR)) } {
        Ok(res) => res,
        Err(_) => return None,
    };

    let ptr = bin_file.as_ptr();

    let func: Symbol<GetMemoryFormatType> = unsafe {
        match lib.get(b"GetMemoryFormat") {
            Ok(res) => res,
            Err(_) => return None,
        }
    };

    let response = func(ptr, bin_file.len() as u32);
    return Some(response);
}

#[cfg(target_os = "windows")]
fn descript_mem_file(bin_file: &Vec<u8>) -> Option<String> {
    let lib = match unsafe { Library::new(format!("{}", DLL_DIR)) } {
        Ok(res) => res,
        Err(_) => return None,
    };

    let ptr = bin_file.as_ptr();

    let func: Symbol<DecodeMemoryType> = unsafe {
        match lib.get(b"DecryptAndDecodeMemory") {
            Ok(res) => res,
            Err(_) => return None,
        }
    };

    let out_buf_size: u32 = 0;
    let out_buf_ptr = &out_buf_size as *const c_uint;

    let response = func(ptr, bin_file.len() as c_uint, 0 as *const u8, out_buf_ptr);

    if response != 0 {
        return None;
    }

    let new_file_data: Vec<u8> = vec![0; out_buf_size as usize];
    let new_file_data_ptr = new_file_data.as_ptr();

    func(
        ptr,
        bin_file.len() as c_uint,
        new_file_data_ptr,
        out_buf_ptr,
    );

    let to_string = match String::from_utf8(new_file_data) {
        Ok(res) => res,
        Err(_) => return None,
    };

    return Some(to_string);
}

#[cfg(target_os = "windows")]
fn descript_3nk_file(bin_file: &Vec<u8>) -> Option<String> {
    let lib = match unsafe { Library::new(format!("{}", DLL_DIR)) } {
        Ok(res) => res,
        Err(_) => return None,
    };

    let ptr = bin_file.as_ptr();

    let func: Symbol<DecodeMemoryType> = unsafe {
        match lib.get(b"DecodeMemory") {
            Ok(res) => res,
            Err(_) => return None,
        }
    };

    let out_buf_size: u32 = 0;
    let out_buf_ptr = &out_buf_size as *const c_uint;

    let response = func(ptr, bin_file.len() as c_uint, 0 as *const u8, out_buf_ptr);

    if response != 0 {
        return None;
    }

    let new_file_data: Vec<u8> = vec![0; out_buf_size as usize];
    let new_file_data_ptr = new_file_data.as_ptr();

    func(
        ptr,
        bin_file.len() as c_uint,
        new_file_data_ptr,
        out_buf_ptr,
    );

    let to_string = match String::from_utf8(new_file_data) {
        Ok(res) => res,
        Err(_) => return None,
    };

    return Some(to_string);
}

#[cfg(target_os = "windows")]
pub async fn decrypt_file(bin_dir: &str) -> Option<String> {
    let bin_file = match read_file_bin(&bin_dir) {
        Some(res) => res,
        None => return None,
    };

    if USE_DECRYPT_TRUCK.load(Ordering::SeqCst) {
        if let Ok(res) = decrypt_truck::decrypt_bin_file(&bin_file) {
            match String::from_utf8(res) {
                Ok(res) => return Some(res),
                // Fallback to SII_Decrypt.dll
                Err(_) => {}
            }
        }
    }

    let memory_format = match get_memory_format(&bin_file) {
        Some(res) => res,
        None => return None,
    };

    match memory_format {
        1 => {
            match String::from_utf8(bin_file) {
                Ok(res) => return Some(res),
                Err(_) => return None,
            };
        }
        2 => {
            match descript_mem_file(&bin_file) {
                Some(res) => return Some(res),
                None => return None,
            };
        }
        4 => {
            match descript_3nk_file(&bin_file) {
                Some(res) => return Some(res),
                None => return None,
            };
        }
        _ => return None,
    }
}

#[cfg(not(target_os = "windows"))]
pub async fn decrypt_file(bin_dir: &str) -> Option<String> {
    let bin_file = match read_file_bin(&bin_dir) {
        Some(res) => res,
        None => return None,
    };

    if let Ok(res) = decrypt_truck::decrypt_bin_file(&bin_file) {
        match String::from_utf8(res) {
            Ok(res) => return Some(res),
            Err(_) => return None,
        }
    }

    None
}

pub async fn decrypt_file_to_save(bin_dir: &str) -> bool {
    let string_file = match decrypt_file(bin_dir).await {
        Some(res) => res,
        None => return false,
    };

    match save_file(&bin_dir.to_string(), &string_file) {
        true => true,
        false => false,
    }
}
