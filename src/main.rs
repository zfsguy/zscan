use std::{
    env,
    ffi::OsString,
    fs::OpenOptions,
    io::{self, ErrorKind, Read, Write},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

const AIM_ON: &[u8] = &[0xc5, 0x04, 0x00];
const AIM_OFF: &[u8] = &[0xc4, 0x04, 0x00];
const ILLUMINATION_ON: &[u8] = &[0xc1, 0x04, 0x00];
const ILLUMINATION_OFF: &[u8] = &[0xc0, 0x04, 0x00];
const CAPABILITIES_REQUEST: &[u8] = &[0xd3, 0x04, 0x00];
const COMMAND_PAUSE: Duration = Duration::from_secs(1);

fn main() -> io::Result<()> {
    let (program_name, device_path) = parse_args();
    let device_path = match device_path {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!(
                "Usage: {} <device_path>",
                Path::new(&program_name).display()
            );
            std::process::exit(2);
        }
    };

    run(&device_path)
}

fn run(device_path: &Path) -> io::Result<()> {
    let mut device = OpenOptions::new()
        .read(true)
        .write(true)
        .open(device_path)?;

    transact(&mut device, AIM_ON, "AIM_ON")?;
    thread::sleep(COMMAND_PAUSE);
    transact(&mut device, AIM_OFF, "AIM_OFF")?;
    transact(&mut device, ILLUMINATION_ON, "ILLUMINATION_ON")?;
    thread::sleep(COMMAND_PAUSE);
    transact(&mut device, ILLUMINATION_OFF, "ILLUMINATION_OFF")?;
    transact(&mut device, CAPABILITIES_REQUEST, "CAPABILITIES_REQUEST")?;

    Ok(())
}

fn transact(device: &mut std::fs::File, command: &[u8], label: &str) -> io::Result<Vec<u8>> {
    let frame = send_command(device, command)?;
    println!("{label} command -> {}", hex_dump(&frame));

    let response = read_packet(device)?;
    println!("{label} response <- {}", hex_dump(&response));

    Ok(response)
}

fn send_command<W: Write>(device: &mut W, command: &[u8]) -> io::Result<Vec<u8>> {
    let frame = frame_command(command);
    device.write_all(&frame)?;
    device.flush()?;
    Ok(frame)
}

fn frame_command(command: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(1 + command.len() + 2);
    frame.push((command.len() + 1) as u8);
    frame.extend_from_slice(command);

    let checksum = checksum_bytes(&frame);
    frame.extend_from_slice(&checksum);

    frame
}

fn read_packet<R: Read>(device: &mut R) -> io::Result<Vec<u8>> {
    let mut length_buf = [0u8; 1];
    device.read_exact(&mut length_buf)?;
    let payload_len = (length_buf[0] as usize) - 1; // Subtract 1 for the length byte itself

    let mut rest = vec![0u8; payload_len + 2];
    device.read_exact(&mut rest)?;

    let mut frame = Vec::with_capacity(1 + rest.len());
    frame.push(length_buf[0]);
    frame.extend_from_slice(&rest);

    validate_checksum(&frame)?;

    Ok(frame)
}

fn validate_checksum(frame: &[u8]) -> io::Result<()> {
    if frame.len() < 3 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "frame too short to contain checksum",
        ));
    }

    let split = frame.len() - 2;
    let (payload, checksum_field) = frame.split_at(split);
    let expected = checksum_bytes(payload);
    let actual = [checksum_field[0], checksum_field[1]];

    if actual != expected {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "checksum mismatch: expected {:02x} {:02x}, got {:02x} {:02x}",
                expected[0], expected[1], actual[0], actual[1]
            ),
        ));
    }

    Ok(())
}

fn checksum_bytes(data: &[u8]) -> [u8; 2] {
    let sum = data
        .iter()
        .fold(0u16, |acc, byte| acc.wrapping_add(*byte as u16));
    let checksum = sum.wrapping_neg();
    checksum.to_be_bytes()
}

fn hex_dump(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_args() -> (OsString, Option<OsString>) {
    let mut args = env::args_os();
    let program_name = args.next().unwrap_or_else(|| OsString::from("zscan"));
    let device_path = args.next();
    (program_name, device_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_matches_reference() {
        let frame = frame_command(&[0xc5, 0x04, 0x00]);
        let (payload, checksum) = frame.split_at(frame.len() - 2);
        assert_eq!(checksum_bytes(payload), [checksum[0], checksum[1]]);
    }

    #[test]
    fn hex_dump_formats_bytes() {
        let formatted = hex_dump(&[0x0f, 0xa0, 0x00]);
        assert_eq!(formatted, "0f a0 00");
    }
}
