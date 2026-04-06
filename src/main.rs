use std::path::PathBuf;
use std::sync::mpsc::*;
use std::thread;
use serial2::SerialPort;

fn main() -> eframe::Result {
    
}

fn poll_port(port_path: PathBuf) -> Receiver<CanMessage> {
    let (tx, rx) = channel();
    
    thread::spawn(|| {
        let port = SerialPort::open(port_path, 500_000).unwrap();
        let mut buffer = [0u8; 256];
        let mut line_buf = Vec::<u8>::new();
        
        loop {
            match port.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    for &byte in &buffer[..n] {
                        if byte != b'\r' && byte != b'\n' {
                            line_buf.push(byte);
                            continue;
                        }
                        
                        if line_buf.is_empty() { continue; }
                        
                        match std::str::from_utf8(&line_buf) {
                            Ok(line) => tx.send(parse_slcan_line(line)).unwrap(),
                            Err(..) => continue
                        }
                        
                        line_buf.clear();
                    }
                },
                Ok(_) => continue,
                Err(..) => panic!("Oopsie")
            }
        }
    });
    
    rx
}

fn parse_slcan_line(line: &str) -> Option<CanMessage> {
    let bytes = line.as_bytes();

    if bytes.first().unwrap() != &b't' { return None; }
    if bytes.len() < 5 { return None; }

    let id_str = &line[1..4];
    let can_id = u16::from_str_radix(id_str, 16).ok().unwrap();

    let dlc = (bytes[4] as char).to_digit(16).unwrap() as usize;
    if dlc > 8 { return None; }

    let mut data = [u8; 8];
    let mut data_start_idx = 5;
    
    for i in 0..dlc {
        if data_start_idx + 2 > bytes.len() { return None };

        let data_byte_str = &line[data_start_idx..data_start_idx + 2];
        let value = u8::from_str_radix(data_byte_str, 16).ok().unwrap();
        data[i] = value;

        data_start_idx += 2;
    }

    Some(CanMessage::new(can_id, data))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanMessage {
    pub id: u16,
    pub dlc: u8,
    pub data: [u8; 8]
}

impl CanMessage {
    pub fn new(id: u16, data: &[u8]) -> Self {
        let mut payload = [0u8; 8];
        payload[..data.len()].copy_from_slice(data);

        Self {
            id,
            dlc: data.len() as u8,
            data: payload
        }
    }
}
