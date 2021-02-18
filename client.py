import socket
from time import sleep

s = socket.create_connection(("bl09csimslm01", 4001))
buff = bytearray(100)
while True:

    s.sendall(b"ver:?\n")
    readed = s.recv_into(buff)
    r = buff.decode("UTF-8")
    print(f"{readed} - {r},")
    sleep(1)
