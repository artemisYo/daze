use std::io::Write;
use std::thread::sleep;

fn main() {
    let mut stream = std::net::TcpStream::connect("127.0.0.1:1984").unwrap();
    stream.write(&[1]);
    stream.write(&[3]);
    stream.write(&("test.dz".len() as u64).to_be_bytes());
    stream.write("test.dz".as_bytes());
}
