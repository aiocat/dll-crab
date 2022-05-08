// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use winapi::shared::minwindef::{BOOL, FARPROC, LPVOID};
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::{FreeLibrary, GetModuleHandleA, GetProcAddress};
use winapi::um::memoryapi::{VirtualAllocEx, VirtualFreeEx, WriteProcessMemory};
use winapi::um::processthreadsapi::{GetExitCodeThread, OpenProcess};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winnt::{
    HANDLE, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, PROCESS_ALL_ACCESS,
};

use std::ffi::{c_void, CString};
use std::mem::transmute;
use std::ptr::null_mut;

// open process wrapper
pub fn open_process(pid: u32) -> Option<HANDLE> {
    let process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false as BOOL, pid) };

    if process.is_null() {
        None
    } else {
        Some(process)
    }
}

// prepare c-compatible string
pub fn c_string(normal_str: &str) -> Option<CString> {
    let new_c_string = CString::new(normal_str);

    match new_c_string {
        Ok(c_string) => Some(c_string),
        Err(_) => None,
    }
}

// alloc adress and write value into it
pub fn write(process: HANDLE, value: CString) -> Option<LPVOID> {
    let adress = unsafe {
        VirtualAllocEx(
            process,
            null_mut(),
            value.as_bytes().len() + 1,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };

    // check if allocted
    if adress.is_null() {
        return None;
    }

    // write to adress
    let mut written_bytes = 0;
    let write_result = unsafe {
        WriteProcessMemory(
            process,
            adress,
            value.as_c_str().as_ptr() as *const c_void,
            value.as_bytes().len() + 1,
            &mut written_bytes,
        )
    };

    // check result
    if write_result == 0 {
        unsafe {
            VirtualFreeEx(process, adress, 0, MEM_RELEASE);
            CloseHandle(process);
        }
        return None;
    }

    Some(adress)
}

pub fn get_load_library() -> FARPROC {
    // get kernel32
    let kernel32_dll = unsafe {
        let kernel32_name = CString::new("kernel32.dll").unwrap();
        GetModuleHandleA(transmute(kernel32_name.as_ptr()))
    };

    // get load library function from kernel32
    let load_library = unsafe {
        let load_library_name = CString::new("LoadLibraryA").unwrap();
        GetProcAddress(kernel32_dll, transmute(load_library_name.as_ptr()))
    };

    // free kernel32
    unsafe { FreeLibrary(kernel32_dll) };

    // return library
    load_library
}

// check process thread and safely close it if it failed
pub fn check_process_thread(thread_process: HANDLE, adress: LPVOID) -> bool {
    if thread_process.is_null() {
        unsafe {
            CloseHandle(thread_process);
            VirtualFreeEx(thread_process, adress, 0, MEM_RELEASE);
        }

        return false;
    }

    true
}

// wait for thread
pub fn wait_for_thread(thread_process: HANDLE) {
    unsafe {
        WaitForSingleObject(thread_process, 0xFFFFFFFF);
    }
}

// check thread exit code
pub fn get_thread_exit_code(thread_process: HANDLE, adress: LPVOID) -> bool {
    let mut exit_code = 0;

    unsafe {
        if GetExitCodeThread(thread_process, &mut exit_code) == false as BOOL {
            CloseHandle(thread_process);
            VirtualFreeEx(thread_process, adress, 0, MEM_RELEASE);

            return false;
        }
    }
    true
}

// safely close process, process thread and adress
pub fn free_process(process: HANDLE, thread_process: HANDLE, adress: LPVOID) -> bool {
    unsafe {
        CloseHandle(thread_process);
        VirtualFreeEx(process, adress, 0, MEM_RELEASE);
        CloseHandle(process);
    };

    true
}

// safely close process and adress
pub fn free_process_without_thread(process: HANDLE, adress: LPVOID) -> bool {
    unsafe {
        VirtualFreeEx(process, adress, 0, MEM_RELEASE);
        CloseHandle(process);
    };

    true
}

// safely close process
pub fn free_process_only(thread_process: HANDLE) -> bool {
    unsafe {
        CloseHandle(thread_process);
    };

    true
}
