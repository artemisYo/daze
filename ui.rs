use std::io::{Write, Read};

fn main() {
    println!("Attempting connection!");
    let mut stream = std::net::TcpStream::connect("127.0.0.1:1984").unwrap();
    stream.write(&[10]);
    let mut vec = vec![0;128];
    stream.read(&mut vec);
    println!("Sent print command!");
}
