extern crate libc;
use libc::{c_uchar, c_uint, c_ulong, c_ulonglong, pipe};
use std::ffi::CString;
use std::os::raw::c_char;
use bytes::Bytes;



#[link(name = "usbdrvd")]
extern {
    fn USBDRVD_GetDevCount(usb_pid: c_ulong) -> c_uint;
    fn USBDRVD_OpenDevice(deviceNumber: c_uint, attributes: c_ulong, usb_pid: c_ulong) -> c_ulonglong;

    fn USBDRVD_CloseDevice(device: c_ulonglong);

    fn USBDRVD_GetPipeCount(device: c_ulonglong) -> c_uint;

    fn USBDRVD_BulkRead(device: c_ulonglong, pipe: c_ulong,  buffer: *const c_char, count: c_ulong) -> c_ulong;

    fn USBDRVD_BulkWrite(device: c_ulonglong, pipe: c_ulong, buffer: *const c_char, count: c_ulong) -> c_ulong;

    fn USBDRVD_PipeOpen(deviceNumber: c_uint, pipe: c_uint, attributes: c_ulong , usb_guid: c_uchar) -> c_ulonglong;

    fn USBDRVD_PipeClose(pipe: c_ulonglong);
}

fn main() {
    println!("Meadowlark Rust USB Virtual COM Port Server!");
    const USB_PID: u16 = 0x139C;
    const flagsandattrs: u32 = 0x40000000;
    let guid = b"\xa2\x2b\x5b\x8b\xc6\x70\x41\x98\x2b\x7d\xfc\x9d\xba\xaa\x85\x93";

    let dev1 = unsafe {USBDRVD_OpenDevice(1, flagsandattrs, USB_PID)};
    let pipe0 = unsafe{USBDRVD_PipeOpen(1, 0, flagsandattrs, guid)};
    let pipe1 = unsafe{USBDRVD_PipeOpen(1, 1, flagsandattrs, guid)};
    println!("dev1: {}, pipe0: {}, pipe1: {}", dev1, pipe0, pipe1);

    
    let mut response : &[u8; 1024];
    unsafe {USBDRVD_BulkWrite(dev1, 1, &buff, 1024)};
    unsafe {USBDRVD_BulkRead(dev1, 0, &_response, 1024)};

    let count = unsafe { USBDRVD_GetDevCount(5020) };

    println!("Devices connected: {}", count)
}
