use std::io::Write;
use std::thread::sleep;

fn main() {
    let mut stream = std::net::TcpStream::connect("127.0.0.1:69").unwrap();
    loop {
	stream.write(&[1]);
	sleep(std::time::Duration::from_millis(500));
    }
}
