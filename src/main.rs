extern crate libc;
use libc::{c_uchar, c_uint, c_ulong, c_ulonglong, size_t};
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

struct Meadowlark{
    usb_pid: u32,
    flagsandattrs: u32,
    guid: [u8;16],

    _dev1: u64,
    _pipe0: u64,
    _pipe1: u64
}

impl Meadowlark {
    fn new(usb_pid: u32, guid: [u8;16]) -> Self {
        Self{usb_pid: usb_pid, flagsandattrs: 0x40000000, guid: guid, _dev1:0, _pipe0: 0, _pipe1: 1}
    }

    fn open(&mut self) {
        self._dev1 =  unsafe{USBDRVD_OpenDevice(1, self.flagsandattrs, self.usb_pid)};
        self._pipe0 = unsafe{USBDRVD_PipeOpen(1, 0, self.flagsandattrs, self.guid.as_ptr())};
        self._pipe1 = unsafe{USBDRVD_PipeOpen(1, 1, self.flagsandattrs, self.guid.as_ptr())};
        println!("dev1: {}, pipe0: {}, pipe1: {}", self._dev1, self._pipe0, self._pipe1);
    }

    fn write(&self, buf: &str, size_r: usize) {
        let c_buf = CString::new(buf).expect("Error creating C String.");
        
        println!("Recv: {:?}, size: {}", c_buf, size_r);
        unsafe {USBDRVD_BulkWrite(self._dev1, 1, c_buf.as_ptr() as *const u8, size_r as u32)};
    }

    fn read_into(&self, buf: &mut String) -> usize {
        let size_r = unsafe {USBDRVD_BulkRead(self._dev1, 0, buf.as_bytes_mut().as_ptr(),
            buf.as_bytes().len() as u64)} as usize;
        return size_r;
    }

    fn close(&self) {
        unsafe {USBDRVD_PipeClose(self._pipe0)};
        unsafe {USBDRVD_PipeClose(self._pipe1)};
        unsafe {USBDRVD_CloseDevice(self._dev1)};
    }
}

const USB_PID: u32 = 0x139C;
const GUID: [u8;16] = [0xa2 ,0x2b ,0x5b ,0x8b ,0xc6 ,0x70 ,0x41 ,0x98 ,0x2b ,0x7d ,0xfc ,0x9d ,0xba ,0xaa ,0x85 ,0x95];

fn handle_client(stream: TcpStream) {


    let mut meadowlark = Meadowlark::new(USB_PID, GUID);
    meadowlark.open();    

    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    let mut buf = String::with_capacity(1024);

    loop {
        match reader.read_line(&mut buf) {
            Ok(size_r) => {

                meadowlark.write(&buf, size_r);
                let size_r = meadowlark.read_into(&mut buf);
                
                println!("Write: {}", buf);
                writer.write_all(&mut buf.as_bytes());
                writer.flush().expect("Error flushing the write buffer.");
            }
            Err(e) => {
                println!("Error reading from the buffer: {}", e);
                meadowlark.close();

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