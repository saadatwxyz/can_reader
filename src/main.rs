use serial2::SerialPort;

fn parse_slcan_line(line: &str) -> Option<(u32, Vec<u8>)> {
    let bytes = line.as_bytes();

    // Must start with 't' for standard data frame
    if bytes.first() != Some(&b't') {
        return None;
    }

    // Need at least: 't' + 3 ID + 1 DLC
    if bytes.len() < 5 {
        return None;
    }

    // Parse 11-bit CAN ID (3 hex chars)
    let id_str = &line[1..4];
    let can_id = u32::from_str_radix(id_str, 16).ok()?;

    // Parse DLC (1 hex char)
    let dlc = (bytes[4] as char).to_digit(16)? as usize;
    if dlc > 8 {
        return None;
    }

    let mut data = Vec::with_capacity(dlc);

    let mut idx = 5;
    for _ in 0..dlc {
        if idx + 2 > bytes.len() {
            return None;
        }

        let byte_str = &line[idx..idx + 2];
        let value = u8::from_str_radix(byte_str, 16).ok()?;
        data.push(value);

        idx += 2;
    }

    Some((can_id, data))
}

fn main() -> std::io::Result<()> {
    // Change this to match your system
    let port_name = "/dev/ttyUSB0"; // e.g. COM3 on Windows
    let baud_rate = 500_000;

    println!("Opening serial port: {} @ {} baud", port_name, baud_rate);

    let port = SerialPort::open(port_name, baud_rate)?;

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
                                if let Some((can_id, data)) = parse_slcan_line(line) {
                                    println!(
                                        "CAN ID: 0x{:X}, Data: {:?}",
                                        can_id, data
                                    );
                                } else {
                                    println!("Unparsed frame: {}", line);
                                }
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
