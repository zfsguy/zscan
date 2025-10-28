use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::fs::File;

// Given a u8 vector, sum the bytes into a signed 16 bit integer and return the 2s complement.
fn checksum(data: &[u8]) -> i16 {
    let checksum: i16 = -(data.iter().map(|b| *b as i16).sum::<i16>());
    checksum
}

// Given a command vector, send it to the device
fn send_command(device: &mut File, command: &[u8]) {
    // Send the command length, the command, and the checksum
    let mut packet = Vec::new();
    packet.push((command.len() + 1) as u8);
    packet.extend_from_slice(command);
    let checksum = checksum(&packet);
    packet.push((checksum >> 8) as u8);
    packet.push((checksum & 0xFF) as u8);

    device.write_all(&packet).unwrap();

    println!("Sent command: {}", packet.iter().map(|b| format!("{:02x} ", b)).collect::<String>());
}

// Given a device, read a packet and return it as a u8 vector
fn read_packet(device: &mut File) -> Vec<u8> {
    let mut buffer = [0; 1];
    device.read_exact(&mut buffer).unwrap();
    let length = buffer[0] as usize + 2 - 1; // Add 2 for the checksum, minus 1 for the lenghth we already read
    let mut packet = vec![0; length];
    device.read_exact(&mut packet).unwrap();
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

    // Create a u8 vector with some data
    let data = vec![0xc5, 0x04, 0x00];
    send_command(&mut device, &data);

    let packet = read_packet(&mut device);
    println!("Received packet: {}", packet.iter().map(|b| format!("{:02x} ", b)).collect::<String>());

    // Sleep for 1 second
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Send a command to the device
    let data = vec![0xc4, 0x04, 0x00];
    send_command(&mut device, &data);

    let packet = read_packet(&mut device);
    println!("Received packet: {}", packet.iter().map(|b| format!("{:02x} ", b)).collect::<String>());
}
