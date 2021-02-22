extern crate libc;
use libc::{c_uchar, c_uint, c_ulong, c_ulonglong, size_t};
use std::{ffi::{ CString, CStr}, io::{BufRead, Read, Write}};
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, LineWriter, Result};

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

struct Meadowlark{
    _dev1: u64,
    _pipe0: u64,
    _pipe1: u64,

    opened: bool
}

impl Meadowlark {
    fn new(usb_pid: u32, guid: [u8;16]) -> Self {
        let flagsandattrs = 0x40000000;
        let _dev1 =  unsafe{USBDRVD_OpenDevice(1, flagsandattrs, usb_pid)};
        let _pipe0 = unsafe{USBDRVD_PipeOpen(1, 0, flagsandattrs, guid.as_ptr())};
        let _pipe1 = unsafe{USBDRVD_PipeOpen(1, 1, flagsandattrs, guid.as_ptr())};

        println!("dev1: {}, pipe0: {}, pipe1: {}", _dev1, _pipe0, _pipe1);
        Self{_dev1: _dev1, _pipe0: _pipe0, _pipe1: _pipe1, opened: false}
    }

    fn close(&mut self) {
        if self.opened {
            unsafe {USBDRVD_PipeClose(self._pipe0)};
            unsafe {USBDRVD_PipeClose(self._pipe1)};
            unsafe {USBDRVD_CloseDevice(self._dev1)};
            self.opened = false;
        }
    }

}

impl Read for &Meadowlark {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let size_r = unsafe {USBDRVD_BulkRead(self._dev1, 0, buf.as_ptr(),buf.len() as u32)} as usize;
        return Ok(size_r);
    }
}

impl Write for &Meadowlark {
    
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let c_buf = CString::new(buf).expect("Error creating C String.");
        
        println!("Recv: {:?}, size: {}", c_buf, buf.len());
        unsafe {USBDRVD_BulkWrite(self._dev1, 1, c_buf.as_ptr() as *const u8, buf.len() as u32)};

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Drop for Meadowlark {

    fn drop(&mut self) {
        self.close();
    }
}
    

    

const USB_PID: u32 = 0x139C;
const GUID: [u8;16] = [0xa2 ,0x2b ,0x5b ,0x8b ,0xc6 ,0x70 ,0x41 ,0x98 ,0x2b ,0x7d ,0xfc ,0x9d ,0xba ,0xaa ,0x85 ,0x95];

fn handle_client(stream: TcpStream) {

    let meadowlark = Meadowlark::new(USB_PID, GUID);  

    let mut tcp_reader = BufReader::new(&stream);
    let mut tcp_writer = LineWriter::new(&stream);

    let mut usb_reader = BufReader::new(&meadowlark);
    let mut usb_writter = LineWriter::new(&meadowlark);

    loop {
        let mut buf = String::with_capacity(1024);
        match tcp_reader.read_line(&mut buf) {
            Ok(size_r) => {
                
                usb_writter.write_all(&buf.as_bytes());
                usb_reader.read_line(&mut buf);
                
                tcp_writer.write_all(&mut buf.as_bytes());
            }
            Err(e) => {
                println!("Error reading from the buffer: {}", e);
                break;
            }
        }
    }
}

fn main() {
    println!("Meadowlark Rust USB Virtual COM Port Server!");
    let address = "0.0.0.0";
    let port = "4001";

    let count = unsafe { USBDRVD_GetDevCount(5020) };
    println!("Devices connected: {}", count);
    if count == 0 {
        println!("No Meadowlark found!");
        return;
    }

    let listener = TcpListener::bind(format!("{}:{}", address, port)).expect("Error lisening at address.");
    println!("Listening at {}:{}", address, port);
    loop {
        // accept connections and process them serially
        for stream in listener.incoming() {
            handle_client(stream.expect("Error obtaining the TCP stream"));
        }
    }

}