use anyhow::{Error, Result};
use log::{debug, warn};
use std::{
    ffi::c_void,
    ptr::{null, null_mut},
};
use windows::{
    core::{s, w}, Wdk::System::{
        SystemInformation::{NtQuerySystemInformation, SYSTEM_INFORMATION_CLASS},
        Threading::{ThreadHideFromDebugger, ZwSetInformationThread},
    }, Win32::{
        Foundation::{CloseHandle, HANDLE, NTSTATUS, STATUS_INFO_LENGTH_MISMATCH, STATUS_SUCCESS},
        System::{LibraryLoader::{GetModuleHandleW, GetProcAddress}, Threading::{
            CreateThread, GetCurrentProcessId, GetCurrentThread, SetThreadPriority,
            WaitForSingleObject, INFINITE, LPTHREAD_START_ROUTINE, THREAD_CREATION_FLAGS,
            THREAD_PRIORITY_LOWEST,
        }},
    }
};

type NtCreateThreadExFunc = unsafe extern "system" fn(
    *mut HANDLE,
    ACCESS_MASK,
    *mut OBJECT_ATTRIBUTES,
    HANDLE,
    *mut extern "system" fn(),
    *mut std::ffi::c_void,
    ULONG,
    ULONG_PTR,
    ULONG_PTR,
    *mut CLIENT_ID,
) -> NTSTATUS;

pub struct DisableDebug{}

impl DisableDebug {
    pub fn create_thread(func: LPTHREAD_START_ROUTINE) -> Result<()> {
        let ntdll = unsafe { GetModuleHandleW(w!("ntdll.dll")) }?;
        let nt_create_thread_ex_warp = unsafe { GetProcAddress(ntdll, s!("NtCreateThreadEx")) };
        if nt_create_thread_ex_warp.is_none() {
            warn!("Get NtCreateThreadEx func address failed");
            return Err(Error::msg("Get NtCreateThreadEx func address failed"));
        }

        let nt_create_thread_ex = unsafe {std::mem::transmute_copy(nt_create_thread_ex_warp.unwrap())};
        unsafe { 
            nt_create_thread_ex()
        }

        Ok(())
    }
}

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

pub const SYSTEM_HANDLE_INFORMATION: SYSTEM_INFORMATION_CLASS = SYSTEM_INFORMATION_CLASS(16);

/// 诱饵线程。当调试器调试进程的时候就会获取所有线程的句柄设置一个空白/特殊的诱饵线程。
/// 通过检查系统句柄表，来判断诱饵进程是否被外部进程(调试器)打开句柄
pub struct HoneyThread {
    pub thread_handle: Option<HANDLE>,
    pub thread_object: *mut c_void,
    pub process_uid: u32,
    pub thread_uid: u32,
}

impl Default for HoneyThread {
    fn default() -> Self {
        Self {
            thread_handle: Default::default(),
            thread_object: null_mut(),
            process_uid: Default::default(),
            thread_uid: Default::default(),
        }
    }
}

impl Drop for HoneyThread {
    fn drop(&mut self) {
        if self.thread_handle.is_some() {
            let _ = unsafe { CloseHandle(self.thread_handle.unwrap()) };
        }

        self.thread_object = null_mut();
        self.process_uid = 0;
        self.thread_uid = 0;
    }
}

impl HoneyThread {
    /// 一个空的线程函数，线程会一直存活直到进程结束
    unsafe extern "system" fn thread_proc(argv: *mut c_void) -> u32 {
        let _ = argv;
        let hthread: HANDLE = unsafe { GetCurrentThread() };
        let _ = unsafe { SetThreadPriority(hthread, THREAD_PRIORITY_LOWEST) };
        unsafe { WaitForSingleObject(hthread, INFINITE) };
        0
    }

    /// 创建一个线程，线程函数需要满足LPTHREAD_START_ROUTINE类型
    ///
    /// # 参数
    ///
    /// - `func`: 线程运行函数，函数声明类型应该为Option<unsafe extern "system" fn(lpthreadparameter: *mut core::ffi::c_void) -> u32>
    /// - `argv`: 函数所需要的参数
    ///
    /// # 返回值
    ///
    /// - 如果创建线程成功则返回Ok(())
    /// - 如果创建线程失败则返回Err
    ///
    /// # 注意
    ///
    /// 这里的独立线程函数，最好保证线程能一只存活，直到进程结束。
    /// 如果线程很快结束的话，可能导致查询系统句柄表时，
    /// 线程已经结束而在判断是否线程被调试时无法正确判断
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut x = HoneyThread::default();
    /// let _ = x.create_thread(HoneyThread::thread_proc, None);
    /// ```
    pub fn create_thread(
        &mut self,
        func: LPTHREAD_START_ROUTINE,
        argv: Option<*const c_void>,
    ) -> Result<()> {
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

        self.thread_handle = Some(hthread);
        self.thread_uid = thread_id;
        Ok(())
    }

    /// 创建一个空白线程，线程能存活到进程结束
    ///
    /// # 返回值
    ///
    /// - 如果创建线程成功则返回Ok(())
    /// - 如果创建线程失败则返回Err
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut x = HoneyThread::default();
    /// let _ = x.create_thread_empty_func().except("create thread failed");
    /// ```
    pub fn create_thread_empty_func(&mut self) -> Result<()> {
        self.create_thread(Some(Self::thread_proc), None)
    }

    /// 查询系统句柄表内容
    ///
    /// # 返回值
    ///
    /// - 如果查询成功返回系统句柄表所有内容，以Vec<u8>的形式
    /// - 如果查询失败则返回报错
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let data = HoneyThread::query_system_information().except("NtQuerySystemInformation failed");
    /// ```
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

    /// 设置空白诱饵线程在当前进程下
    ///
    /// # 返回值
    ///
    /// - 如果设置成功则返回Ok(())
    /// - 如果设置失败则返回Err
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut x = HoneyThread::default();
    /// let _ = x.set_honey_thread_current_process();
    /// ```
    pub fn set_honey_thread_current_process(&mut self) -> Result<()> {
        self.create_thread_empty_func()?;
        self.process_uid = unsafe { GetCurrentProcessId() };

        debug!(
            "hthread value ==> {:?}; process id ==> {}",
            self.thread_handle, self.process_uid
        );

        Ok(())
    }

    /// 判断指定进程的句柄是否被其他进程获取
    ///
    /// - 通过遍历判断句柄值与进程号来找到对应句柄的内核地址值
    /// - 通过对比内核地址值与进程号来判断，改句柄是否被其他进程打开
    ///
    /// # 返回值
    ///
    /// - `Ok(true)`: 句柄被其他进程获取
    /// - `Ok(false)`: 句柄未被其他进程获取
    /// - `Err`: 系统函数执行报错或者系统句柄表中未找到指定句柄内核地址
    ///
    /// # 注意
    ///
    /// 调用示例的check函数，需要已经初始化了self.thread_handle和self.process_uid值
    /// 可以调用set_honey_thread_current_process来设置空白线程句柄与当前进程ID
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut x = HoneyThread::default();
    /// let _ = x.set_honey_thread_current_process();
    /// assert_eq!(x.check().unwarp(), false);
    /// ```
    pub fn check(&mut self) -> Result<bool> {
        if self.thread_handle.is_none() || self.process_uid == 0 {
            warn!(
                "HoneyThread instance value error!; please set the process uid and thread handle"
            );
            return Err(Error::msg(
                "HoneyThread instance value error!; please set the process uid and thread handle",
            ));
        }

        // 获取系统句柄表信息
        let system_information: Vec<u8> = Self::query_system_information()?;
        let handle_info: *const SystemHandleInformation =
            system_information.as_ptr() as *const SystemHandleInformation;
        let handle_info_ref: &SystemHandleInformation = unsafe { &*handle_info };
        let handles_ptr: *const SystemHandleTableEntryInfo = handle_info_ref.handles.as_ptr();

        if self.thread_object.is_null() {
            // 获取当前线程内核对象地址
            for i in 0..handle_info_ref.number_of_handles {
                let handle: *const SystemHandleTableEntryInfo =
                    unsafe { handles_ptr.add(i as usize) };
                let uid: u32 = unsafe { (*handle).unique_process_id }.into();
                let handle_val: usize = unsafe { (*handle).handle_value }.into();

                if uid == self.process_uid && handle_val == self.thread_handle.unwrap().0 as usize {
                    self.thread_object = unsafe { (*handle).object };
                    debug!("Get current thread object ==> {:p}", self.thread_object);
                }
            }
        }

        if self.thread_object.is_null() {
            warn!("Could't found currnet thread object");
            return Err(Error::msg("Could't found currnet thread object"));
        }

        // 对比所有内核地址，判断是否存在其他进程也获取了对应的线程内核对象
        for i in 0..handle_info_ref.number_of_handles {
            let handle: *const SystemHandleTableEntryInfo = unsafe { handles_ptr.add(i as usize) };
            let uid: u32 = unsafe { (*handle).unique_process_id }.into();

            if uid == self.process_uid {
                continue;
            }

            let object_addr = unsafe { (*handle).object } as usize;
            if self.thread_object as usize == object_addr {
                debug!("Found attack process is debug ==> {:?}", unsafe {
                    &*handle
                });
                return Ok(true);
            }
        }

        Ok(false)
    }
}
