extern crate libc;
use libc::{c_uchar, c_uint, c_ulong, c_ulonglong};
use std::ffi::{ CString, CStr};


#[link(name = "usbdrvd")]
extern {
    fn USBDRVD_GetDevCount(usb_pid: c_ulong) -> c_uint;
    fn USBDRVD_OpenDevice(deviceNumber: c_uint, attributes: c_ulong, usb_pid: c_ulong) -> c_ulonglong;

    fn USBDRVD_CloseDevice(device: c_ulonglong);

    //fn USBDRVD_GetPipeCount(device: c_ulonglong) -> c_uint;

    fn USBDRVD_BulkRead(device: c_ulonglong, pipe: c_ulong,  buffer: *const c_uchar, count: c_ulong) -> c_ulong;

    fn USBDRVD_BulkWrite(device: c_ulonglong, pipe: c_ulong, buffer: *const c_uchar, count: c_ulong) -> c_ulong;

    fn USBDRVD_PipeOpen(deviceNumber: c_uint, pipe: c_uint, attributes: c_ulong , usb_guid: *const c_uchar) -> c_ulonglong;

    fn USBDRVD_PipeClose(pipe: c_ulonglong);
}

fn main() {
    println!("Meadowlark Rust USB Virtual COM Port Server!");
    const USB_PID: u32 = 0x139C;
    let flagsandattrs: u32 = 0x40000000;
    const GUID: &[u8;16] = b"\xa2\x2b\x5b\x8b\xc6\x70\x41\x98\x2b\x7d\xfc\x9d\xba\xaa\x85\x95";

    let count = unsafe { USBDRVD_GetDevCount(5020) };
    println!("Devices connected: {}", count);
    if count == 0 {
        println!("No Meadowlark found!");
        return;
    }

    let dev1 =  unsafe{USBDRVD_OpenDevice(1, flagsandattrs, USB_PID)};
    let pipe0 = unsafe{USBDRVD_PipeOpen(1, 0, flagsandattrs, GUID.as_ptr())};
    let pipe1 = unsafe{USBDRVD_PipeOpen(1, 1, flagsandattrs, GUID.as_ptr())};
    println!("dev1: {}, pipe0: {}, pipe1: {}", dev1, pipe0, pipe1);

    
    let response : [u8; 1024] = [0 as u8; 1024];
    let cs: [u8; 7] = [b'v', b'e', b'r', b':', b'?', b'\n', b'\0'];
    //let _buff = CString::new(cs).expect("CString::new failed");
    unsafe {USBDRVD_BulkWrite(dev1, 1, cs.as_ptr(), 7)};
    unsafe {USBDRVD_BulkRead(dev1, 0, response.as_ptr(), 1024)};

    let rust_response: &CStr = unsafe { CStr::from_ptr(response.as_ptr() as *const i8 ) };
    println!("Response: {}", rust_response.to_str().unwrap());

    unsafe {USBDRVD_PipeClose(pipe0)};
    unsafe {USBDRVD_PipeClose(pipe1)};
    unsafe {USBDRVD_CloseDevice(dev1)};
}
