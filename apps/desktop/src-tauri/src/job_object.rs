use std::os::windows::prelude::AsRawHandle;
use std::process::Child;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};


pub struct JobObject {
    handle: HANDLE,
}

impl JobObject {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let handle = CreateJobObjectW(None, None).map_err(|e| e.to_string())?;

            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            if let Err(e) = SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) {
                let _ = CloseHandle(handle);
                return Err(e.to_string());
            }

            Ok(Self { handle })
        }
    }

    pub fn add_process(&self, child: &Child) -> Result<(), String> {
        unsafe {
            let process_handle = HANDLE(child.as_raw_handle() as isize);
            AssignProcessToJobObject(self.handle, process_handle).map_err(|e| e.to_string())
        }
    }
}

impl Drop for JobObject {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
