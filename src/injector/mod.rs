// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT
#![allow(clippy::missing_safety_doc)]

mod wrapper;
use winapi::shared::minwindef::{BOOL, DWORD};
use winapi::shared::ntdef::NT_SUCCESS;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{CreateRemoteThread, OpenThread, QueueUserAPC};
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use winapi::um::winnt::{
    HANDLE, PHANDLE, THREAD_GET_CONTEXT, THREAD_SET_CONTEXT, THREAD_SUSPEND_RESUME,
};

use ntapi::ntpsapi::NtCreateThreadEx;
use ntapi::ntrtl::RtlCreateUserThread;

use std::mem;
use std::ptr;

// inject dll with CreateRemoteThread method
pub fn inject_create_remote_thread(pid: u32, dll_path: &str) -> bool {
    // c-compatible string for injecting to process memory
    let path_to_dll = wrapper::c_string(dll_path);
    if path_to_dll.is_none() {
        return false;
    }
    let path_to_dll = path_to_dll.unwrap(); // shadowing

    // get process
    let process = wrapper::open_process(pid);
    if process.is_none() {
        return false;
    }
    let process = process.unwrap(); // shadowing

    // alloc adress and write it for dll path
    let adress = wrapper::write(process, path_to_dll);
    if adress.is_none() {
        return false;
    }
    let adress = adress.unwrap(); // shadowing

    // get load library
    let load_library = wrapper::get_load_library();

    // load dll
    let mut thread_id = 0;
    let thread_process = unsafe {
        CreateRemoteThread(
            process,
            ptr::null_mut(),
            0,
            Some(mem::transmute(load_library)),
            adress,
            0,
            &mut thread_id,
        )
    };

    // check if dll loaded
    if thread_process.is_null() {
        wrapper::free_process(process, thread_process, adress);
        return false;
    }

    // check status and wait for thread and get thread exit code
    if !wrapper::check_process_thread(thread_process, adress) {
        return false;
    }

    wrapper::wait_for_thread(thread_process);

    if !wrapper::get_thread_exit_code(thread_process, adress) {
        return false;
    }

    // de-alloc memory, free libraries (memory safety)
    wrapper::free_process(process, thread_process, adress)
}

// inject dll with RtlCreateUserThread function which is undocumented
pub fn inject_rtl_create_user_thread(pid: u32, dll_path: &str) -> bool {
    // c-compatible string for injecting to process memory
    let path_to_dll = wrapper::c_string(dll_path);
    if path_to_dll.is_none() {
        return false;
    }
    let path_to_dll = path_to_dll.unwrap(); // shadowing

    // get process
    let process = wrapper::open_process(pid);
    if process.is_none() {
        return false;
    }
    let process = process.unwrap(); // shadowing

    // alloc adress and write it for dll path
    let adress = wrapper::write(process, path_to_dll);
    if adress.is_none() {
        return false;
    }
    let adress = adress.unwrap(); // shadowing

    // get load library
    let load_library = wrapper::get_load_library();

    // load dll
    let mut thread_process: HANDLE = ptr::null_mut();
    let thread_result: bool;

    unsafe {
        thread_result = NT_SUCCESS(RtlCreateUserThread(
            process,
            ptr::null_mut(),
            0,
            0,
            0,
            0,
            Some(mem::transmute(load_library)),
            adress,
            &mut thread_process as PHANDLE,
            ptr::null_mut(),
        ));
    }

    // check if dll loaded
    if !thread_result {
        wrapper::free_process(process, thread_process, adress);
        return false;
    }

    // check status and wait for thread and get thread exit code
    if !wrapper::check_process_thread(thread_process, adress) {
        return false;
    }

    wrapper::wait_for_thread(thread_process);

    if !wrapper::get_thread_exit_code(thread_process, adress) {
        return false;
    }

    // de-alloc memory, free libraries (memory safety)
    wrapper::free_process(process, thread_process, adress)
}

// inject dll with QueueUserAPC method
pub fn inject_queue_user_apc(pid: u32, dll_path: &str) -> bool {
    // c-compatible string for injecting to process memory
    let path_to_dll = wrapper::c_string(dll_path);
    if path_to_dll.is_none() {
        return false;
    }
    let path_to_dll = path_to_dll.unwrap(); // shadowing

    // get process
    let process = wrapper::open_process(pid);
    if process.is_none() {
        return false;
    }
    let process = process.unwrap(); // shadowing

    // get tids
    let (tids, success) = unsafe { get_tids_by_pid(pid) };
    if !success {
        wrapper::free_process_only(process);
        return false;
    }

    // alloc adress and write it for dll path
    let adress = wrapper::write(process, path_to_dll);
    if adress.is_none() {
        return false;
    }
    let adress = adress.unwrap(); // shadowing

    // get load library
    let load_library = wrapper::get_load_library();

    // load dll to all threads
    for tid in &tids {
        let thread_process = unsafe {
            OpenThread(
                THREAD_SET_CONTEXT | THREAD_GET_CONTEXT | THREAD_SUSPEND_RESUME,
                false as BOOL,
                *tid,
            )
        };

        // check status
        if thread_process.is_null() {
            wrapper::free_process_only(thread_process);
            continue;
        }

        // inject
        let thread_result = unsafe {
            QueueUserAPC(
                Some(mem::transmute(load_library)),
                thread_process,
                adress as usize,
            ) != 0
        };

        // check result
        if !thread_result {
            wrapper::free_process_only(thread_process);
            continue;
        } else {
            // wait for thread
            wrapper::wait_for_thread(thread_process);

            // get thread exit result
            if !wrapper::get_thread_exit_code(thread_process, adress) {
                wrapper::free_process_only(thread_process);
            }
        }
    }

    wrapper::free_process_without_thread(process, adress)
}

// inject dll with NtCreateThreadEx function which is undocumented
pub fn inject_nt_create_thread_ex(pid: u32, dll_path: &str) -> bool {
    // c-compatible string for injecting to process memory
    let path_to_dll = wrapper::c_string(dll_path);
    if path_to_dll.is_none() {
        return false;
    }
    let path_to_dll = path_to_dll.unwrap(); // shadowing

    // get process
    let process = wrapper::open_process(pid);
    if process.is_none() {
        return false;
    }
    let process = process.unwrap(); // shadowing

    // alloc adress and write it for dll path
    let adress = wrapper::write(process, path_to_dll);
    if adress.is_none() {
        return false;
    }
    let adress = adress.unwrap(); // shadowing

    // get load library
    let load_library = wrapper::get_load_library();

    // load dll
    let mut thread_process: HANDLE = ptr::null_mut();
    let thread_result: bool;

    unsafe {
        thread_result = NT_SUCCESS(NtCreateThreadEx(
            &mut thread_process,
            0x1FFFFF,
            ptr::null_mut(),
            process,
            mem::transmute(load_library),
            adress,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
        ));
    }

    // check if dll loaded
    if !thread_result {
        wrapper::free_process(process, thread_process, adress);
        return false;
    }

    // check status and wait for thread and get thread exit code
    if !wrapper::check_process_thread(thread_process, adress) {
        return false;
    }

    wrapper::wait_for_thread(thread_process);

    if !wrapper::get_thread_exit_code(thread_process, adress) {
        return false;
    }

    // de-alloc memory, free libraries (memory safety)
    wrapper::free_process(process, thread_process, adress)
}

// list tids from pid
unsafe fn get_tids_by_pid(pid: u32) -> (Vec<DWORD>, bool) {
    let mut tids: Vec<DWORD> = Vec::new();

    let mut entry: THREADENTRY32 = THREADENTRY32 {
        dwSize: 0_u32,
        cntUsage: 0_u32,
        th32ThreadID: 0_u32,
        th32OwnerProcessID: 0_u32,
        tpBasePri: 0,
        tpDeltaPri: 0,
        dwFlags: 0_u32,
    };
    entry.dwSize = mem::size_of_val(&entry) as u32;

    let handle_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);

    if Thread32First(handle_snapshot, &mut entry) != 0 {
        loop {
            if pid == entry.th32OwnerProcessID {
                tids.push(entry.th32ThreadID);
            }

            if Thread32Next(handle_snapshot, &mut entry) == 0 {
                break;
            }
        }
    }

    if tids.is_empty() {
        return (tids, false);
    }

    CloseHandle(handle_snapshot);
    (tids, true)
}
