extern crate libc;
use libc::{c_uchar, c_uint, c_ulong, c_ulonglong};
use std::{ffi::{ CString, CStr}, io::{BufRead, Write}};
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, BufWriter};

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

fn handle_client(stream: TcpStream) {
    const USB_PID: u32 = 0x139C;
    let flagsandattrs: u32 = 0x40000000;
    const GUID: &[u8;16] = b"\xa2\x2b\x5b\x8b\xc6\x70\x41\x98\x2b\x7d\xfc\x9d\xba\xaa\x85\x95";

    let dev1 =  unsafe{USBDRVD_OpenDevice(1, flagsandattrs, USB_PID)};
    let pipe0 = unsafe{USBDRVD_PipeOpen(1, 0, flagsandattrs, GUID.as_ptr())};
    let pipe1 = unsafe{USBDRVD_PipeOpen(1, 1, flagsandattrs, GUID.as_ptr())};
    println!("dev1: {}, pipe0: {}, pipe1: {}", dev1, pipe0, pipe1);

    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    loop {
        let mut buf = String::with_capacity(1024);
        match reader.read_line(&mut buf) {
            Ok(size_r) => {
                let c_buf = CString::new(buf).expect("Error creating C String.");
        
                println!("Recv: {:?}, size: {}", c_buf, size_r);
                unsafe {USBDRVD_BulkWrite(dev1, 1, c_buf.as_ptr() as *const u8, size_r as u32)};

                let response : [u8; 1024] = [0 as u8; 1024];
                unsafe {USBDRVD_BulkRead(dev1, 0, response.as_ptr(), 1024)};
                let rust_response: &CStr = unsafe { CStr::from_ptr(response.as_ptr() as *const i8 ) };
                println!("Response: {}", rust_response.to_str().unwrap());

                writer.write_all(&response).expect("Error sending response throught socket.");
                writer.flush();
            }
            Err(e) => {
                println!("Error reading from the buffer: {}", e);
                unsafe {USBDRVD_PipeClose(pipe0)};
                unsafe {USBDRVD_PipeClose(pipe1)};
                unsafe {USBDRVD_CloseDevice(dev1)};

                break;
            }
        }
    }
}

fn main() {
    println!("Meadowlark Rust USB Virtual COM Port Server!");
    let address = "127.0.0.1";
    let port = "4001";

    let count = unsafe { USBDRVD_GetDevCount(5020) };
    println!("Devices connected: {}", count);
    if count == 0 {
        println!("No Meadowlark found!");
        return;
    }

    let listener = TcpListener::bind(format!("{}:{}", address, port)).expect("Error lisening at address.");
    println!("Listening at {}:{}", address, port);
    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream.expect("Error obtaining the TCP stream"));
    }
}
