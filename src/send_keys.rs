use std::{mem, thread::sleep, time::Duration};
use winapi::um::winuser::{SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP};
// key codes: https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes

unsafe fn create_input(key_code: u16, flags: u32) -> INPUT {
    let mut input = mem::zeroed::<INPUT>();
    input.type_ = INPUT_KEYBOARD;
    let mut ki = input.u.ki_mut();
    ki.wVk = key_code;
    ki.dwFlags = flags;
    input
}

unsafe fn key_down(key_code: u16) {
    let mut input = create_input(key_code, 0);
    SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
}

unsafe fn key_up(key_code: u16) {
    let mut input = create_input(key_code, KEYEVENTF_KEYUP);
    SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
}

pub unsafe fn key_enter(key_code: u16) {
    key_down(key_code);
    sleep(Duration::from_millis(154));
    key_up(key_code);
}
