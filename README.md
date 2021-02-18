# Meadowlark USB Virtual COM Port Socket Server in rust

This server is a proxi that exposes the USB comunication of a Meadowlark D5020 polarizer throught a TCP socket.

Meadowlark usbdrvd.lib and usbdrvd.dll x64 are required to build and run this software.

## How to build
```
cargo build --release
```

## How to run
```
cargo run --release
```

## Test comunication

```
import socket
from time import sleep

s = socket.create_connection(("localhost", 4001))
buff = bytearray(100)
while True:

    s.sendall(b"ver:?\n")
    readed = s.recv_into(buff)
    r = buff.decode("UTF-8")
    print(f"{readed} - {r},")
    sleep(1)
```