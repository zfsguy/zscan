use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::fs::File;
use std::thread;
use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Command {
    AimOn = 0xc5,
    AimOff = 0xc4,
    IlluminationOn = 0xc1,
    IlluminationOff = 0xc0,
    CapabilitiesRequest = 0xd3,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Source {
    Device = 0x00,
    Host = 0x04,
}

pub struct Packet {
    pub length: u8,
    pub command: Command,
    pub source: Source,
    pub status: u8,
    pub payload: Vec<u8>,
    pub checksum: i16,
}

fn hex_dump(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<_>>()
        .join(" ")
}

// Given a u8 vector, sum the bytes into a signed 16 bit integer and return the 2s complement.
fn checksum(data: &[u8]) -> i16 {
    let checksum: i16 = data.iter().map(|b| *b as i16).sum::<i16>();
    -checksum
}

fn compose_packet(command: Command, payload: &[u8]) -> Vec<u8> {
    // The length of the packet is the payload, plus header, plus checksum
    let len = payload.len() + 4 + 2; // 4 for the header, 2 for the checksum
    let mut packet = Vec::with_capacity(len);
    packet.push((len - 2) as u8); // Subtract 2 for the checksum
    packet.push(command as u8);
    packet.push(Source::Host as u8);
    packet.push(0); // Status is always 0
    packet.extend_from_slice(payload);
    let checksum = checksum(&packet);
    packet.push((checksum >> 8) as u8);
    packet.push((checksum & 0xFF) as u8);
    packet
}

// Given a command vector, send it to the device
fn send_command(device: &mut File, command: Command, payload: &[u8]) -> Vec<u8> {
    let packet = compose_packet(command, payload);
    device.write_all(&packet).unwrap();
    packet
}

// Given a device, read a packet and return it as a u8 vector
fn read_packet(device: &mut File) -> Vec<u8> {
    let mut length_byte = [0; 1];
    device.read_exact(&mut length_byte).unwrap();

    // length_byte[0] = command + source + status + payload (doesn't include checksum)
    let total_length = length_byte[0] as usize + 2;
    let mut packet = vec![0; total_length];
    packet[0] = length_byte[0];

    device.read_exact(&mut packet[1..]).unwrap();
    packet
}

fn transact_command(device: &mut File, command: Command, payload: &[u8]) -> Vec<u8> {
    let packet = send_command(device, command, payload);
    println!("{:?} -> {}", command, hex_dump(&packet));
    let packet = read_packet(device);
    println!("{:?} <- {}", command, hex_dump(&packet));
    packet
}

fn main() {
    println!("Hello, world!");
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <device_path>", args[0]);
        return;
    }
    let device_path = &args[1];
    let mut device = OpenOptions::new()
        .read(true)
        .write(true)
        .open(device_path)
        .unwrap();

    transact_command(&mut device, Command::AimOn, &[]);
    thread::sleep(Duration::from_secs(1));
    transact_command(&mut device, Command::AimOff, &[]);
    transact_command(&mut device, Command::CapabilitiesRequest, &[]);
}
