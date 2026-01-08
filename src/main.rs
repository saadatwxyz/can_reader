use serial2::SerialPort;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    // Change this to match your system
    let port_name = "/dev/ttyUSB0"; // e.g. COM3 on Windows
    let baud_rate = 500_000;

    println!("Opening serial port: {} @ {} baud", port_name, baud_rate);

    let mut port = SerialPort::open(port_name, baud_rate)?;
    port.set_read_timeout(Duration::from_millis(100))?;

    let mut buffer = [0u8; 256];
    let mut line_buf = Vec::<u8>::new();

    println!("Listening for CAN frames...\n");

    loop {
        match port.read(&mut buffer) {
            Ok(n) if n > 0 => {
                for &byte in &buffer[..n] {
                    // SLCAN frames typically end with '\r'
                    if byte == b'\r' || byte == b'\n' {
                        if !line_buf.is_empty() {
                            if let Ok(line) = std::str::from_utf8(&line_buf) {
                                // Just dump raw frame text for now
                                println!("CAN: {}", line);
                            } else {
                                println!("CAN (non-UTF8): {:?}", line_buf);
                            }
                            line_buf.clear();
                        }
                    } else {
                        line_buf.push(byte);
                    }
                }
            }
            Ok(_) => {
                // No data read, timeout hit
            }
            Err(e) => {
                eprintln!("Serial read error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
