use std::ptr::null_mut;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcessToken;
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY};

pub fn is_running_as_admin() -> bool {
    unsafe {
        let mut token_handle: HANDLE = null_mut();
        let process_handle = winapi::um::processthreadsapi::GetCurrentProcess();
        if OpenProcessToken(process_handle, TOKEN_QUERY, &mut token_handle) == 0 {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0;
        let result = GetTokenInformation(
            token_handle,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );
        CloseHandle(token_handle);
        if result == 0 {
            return false;
        }
        elevation.TokenIsElevated != 0
    }
}
