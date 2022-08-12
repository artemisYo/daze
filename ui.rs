use std::io::{Write, Read};

fn main() {
    println!("Attempting connection!");
    let mut stream = std::net::TcpStream::connect("127.0.0.1:1984").unwrap();
    stream.write(&[10]);
    println!("Sent print command!");
}
