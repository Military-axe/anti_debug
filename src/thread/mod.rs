use anyhow::{Error, Result};
use log::{debug, warn};
use std::{ffi::c_void, ptr::{null, null_mut}};
use windows::{
    Wdk::System::{
        SystemInformation::{NtQuerySystemInformation, SYSTEM_INFORMATION_CLASS},
        Threading::{ThreadHideFromDebugger, ZwSetInformationThread},
    },
    Win32::{
        Foundation::{
            HANDLE, NTSTATUS,
            STATUS_INFO_LENGTH_MISMATCH, STATUS_SUCCESS,
        },
        System::Threading::{
            CreateThread, GetCurrentProcessId, GetCurrentThread,
            SetThreadPriority, WaitForSingleObject, INFINITE, LPTHREAD_START_ROUTINE,
            THREAD_CREATION_FLAGS, THREAD_PRIORITY_LOWEST,
        },
    },
};

/// 禁止指定线程调试事件，如果已经在调试中，则会关闭调试
///
/// # 返回值
///
/// - `Err`: ZwSetInformationThread API调用失败
/// - `Ok(())`: 成功禁止线程调试
pub fn disable_thread_debug(hthread: HANDLE) -> Result<()> {
    let status: NTSTATUS =
        unsafe { ZwSetInformationThread(hthread, ThreadHideFromDebugger, null(), 0) };

    if status != STATUS_SUCCESS {
        warn!("ZwSetInformationThread failed; error code: {:?}", status);
        return Err(Error::msg("ZwSetInformationThread failed"));
    }

    Ok(())
}

/// 禁止当前线程调试事件生成，如果已经在调试中，则会关闭调试
///
/// # 返回值
///
/// - `Err`: disable_thread_debug 函数调用失败
/// - `Ok(())`: 成功禁止当前线程调试
pub fn disable_current_thread_debug() -> Result<()> {
    let hthread: HANDLE = unsafe { GetCurrentThread() };
    disable_thread_debug(hthread)
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SystemHandleTableEntryInfo {
    pub unique_process_id: u16,
    pub creator_back_trace_index: u16,
    pub object_type_index: u8,
    pub handle_attributes: u8,
    pub handle_value: u16,
    pub object: *mut c_void,
    pub granted_access: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SystemHandleInformation {
    pub number_of_handles: u32,
    pub handles: [SystemHandleTableEntryInfo; 1],
}

pub struct HoneyThread {}
pub const SYSTEM_HANDLE_INFORMATION: SYSTEM_INFORMATION_CLASS = SYSTEM_INFORMATION_CLASS(16);

impl HoneyThread {
    unsafe extern "system" fn thread_proc(argv: *mut c_void) -> u32 {
        let _ = argv;
        let hthread: HANDLE = unsafe { GetCurrentThread() };
        let _ = unsafe { SetThreadPriority(hthread, THREAD_PRIORITY_LOWEST) };
        unsafe { WaitForSingleObject(hthread, INFINITE) };
        0
    }

    pub fn create_thread(
        func: LPTHREAD_START_ROUTINE,
        argv: Option<*const c_void>,
    ) -> Result<(HANDLE, u32)> {
        let mut thread_id: u32 = Default::default();
        let hthread: HANDLE = unsafe {
            CreateThread(
                None,
                0,
                func,
                argv,
                THREAD_CREATION_FLAGS(0),
                Some(&mut thread_id),
            )
        }?;

        debug!("hthread ==> {:?}; thread_id ==> {:?}", hthread, thread_id);

        Ok((hthread, thread_id))
    }

    pub fn create_thread_empty_func() -> Result<(HANDLE, u32)> {
        Self::create_thread(Some(Self::thread_proc), None)
    }

    pub fn query_system_information() -> Result<Vec<u8>> {
        let mut handle_info_size: usize = 0x10000;
        let mut handle_info_buffer: Vec<u8> = Vec::with_capacity(handle_info_size);
        let mut status: NTSTATUS = STATUS_INFO_LENGTH_MISMATCH;
        let mut return_length: u32 = 0;

        while status == STATUS_INFO_LENGTH_MISMATCH {
            handle_info_buffer.clear();
            handle_info_size = return_length as usize;
            handle_info_buffer.reserve(handle_info_size);
            status = unsafe {
                NtQuerySystemInformation(
                    SYSTEM_HANDLE_INFORMATION,
                    handle_info_buffer.as_mut_ptr() as *mut c_void,
                    handle_info_size as u32,
                    &mut return_length,
                )
            };
        }

        if status != STATUS_SUCCESS {
            warn!("NtQuerySystemInformation failed! status: {:?}", status);
            return Err(Error::msg("NtQuerySystemInformation failed!"));
        }

        debug!("NtQuerySystemInformation query system handle infomation successfully");

        Ok(handle_info_buffer)
    }

    fn check(hthread: HANDLE, process_uid: u32) -> Result<bool> {
        let system_information: Vec<u8> = Self::query_system_information()?;
        let handle_info: *const SystemHandleInformation =
            system_information.as_ptr() as *const SystemHandleInformation;
        let handle_info_ref: &SystemHandleInformation = unsafe { &*handle_info };
        let handles_ptr: *const SystemHandleTableEntryInfo = handle_info_ref.handles.as_ptr();

        let mut current_thread_obj: *mut c_void = null_mut();

        // 获取当前线程内核对象地址
        for i in 0..handle_info_ref.number_of_handles {
            let handle: *const SystemHandleTableEntryInfo = unsafe { handles_ptr.add(i as usize) };
            let uid: u32 = unsafe { (*handle).unique_process_id }.into();
            let handle_val: usize = unsafe { (*handle).handle_value }.into();

            if uid == process_uid && handle_val == hthread.0 as usize {
                current_thread_obj = unsafe { (*handle).object };
                debug!("Get current thread object ==> {:p}", current_thread_obj);
            }
        }

        if current_thread_obj.is_null() {
            warn!("Could't found currnet thread object");
            return Err(Error::msg("Could't found currnet thread object"));
        }

        // 对比所有内核地址，判断是否存在其他进程也获取了对应的线程内核对象
        for i in 0..handle_info_ref.number_of_handles {
            let handle: *const SystemHandleTableEntryInfo = unsafe { handles_ptr.add(i as usize) };
            let uid: u32 = unsafe { (*handle).unique_process_id }.into();

            if uid == process_uid {
                continue;
            }

            let object_addr = unsafe {(*handle).object} as usize;
            if current_thread_obj as usize == object_addr {
                debug!("Found attack process is debug ==> {:?}", unsafe{&*handle});
                return Ok(true)
            }
        }

        Ok(false)
    }

    pub fn honey_thread_current_process() -> Result<bool> {
        let (hthread, _) = Self::create_thread_empty_func()?;
        let process_uid = unsafe {GetCurrentProcessId()};
        
        debug!("hthread value ==> {:?}; process id ==> {}", hthread, process_uid);

        Self::check(hthread, process_uid)
    }
}
