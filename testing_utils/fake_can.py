#!/usr/bin/env python3
import os
import pty
import time
import select

def make_slcan_frame(can_id, data):
    can_id_str = f"{can_id:03X}"
    dlc = len(data)
    data_str = "".join(f"{b:02X}" for b in data)
    return f"t{can_id_str}{dlc}{data_str}\r".encode()

def main():
    master_fd, slave_fd = pty.openpty()
    slave_name = os.ttyname(slave_fd)

    print("Fake SLCAN device ready")
    print(f"On: {slave_name}")

    os.set_blocking(master_fd, False)

    counter = 0
    last_tx = time.time()

    while True:
        # Handle incoming commands from the app
        r, _, _ = select.select([master_fd], [], [], 0.1)
        if master_fd in r:
            try:
                data = os.read(master_fd, 1024)
                if data:
                    print("RX:", data.strip())
                    # Acknowledge common SLCAN commands
                    if data[:1] in b"OSC":
                        os.write(master_fd, b"\r")
            except BlockingIOError:
                pass

        # Periodically send a CAN frame
        if time.time() - last_tx > 1.0:
            payload = [(counter + i) & 0xFF for i in range(8)]
            frame = make_slcan_frame(0x123, payload)
            os.write(master_fd, frame)
            print("TX:", frame.strip())
            counter += 1
            last_tx = time.time()

if __name__ == "__main__":
    main()

