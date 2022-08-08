use std::net::TcpStream;
use std::io::{Write, Read};

// Implemented Instructions:
// 1. Open
// 2. Close
// 3. Save
// 4. Print

// TODO:

fn main() -> std::io::Result<()> {
    let mut stream: TcpStream;
    if let Ok(default_attempt) = TcpStream::connect("127.0.0.1:69") {
        stream = default_attempt;
    } else {
        let mut tmp = String::new();
        println!("Please input the servers ip:");
        std::io::stdin().read_line(&mut tmp)?;
        stream = TcpStream::connect(tmp.trim())?;
    }
    'something: loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let tokens = line.split_whitespace().collect::<Vec<&str>>();
        for t in 0..tokens.len() {
            match tokens[t] {
                ".connect" => {
                    stream.shutdown(std::net::Shutdown::Both)?;
                    let ip = tokens[t+1];
                    stream = TcpStream::connect(ip)?;
                },
                ".close" => {
                    println!("Closing!");
                    stream.write(&[255])?;
                },
                "quit" => {
                    println!("Quitting!");
                    stream.write(&[255])?;
                    break 'something
                },
                "open" => {
                    let filename = tokens[t+1];
                    stream.write(&[3])?;
                    stream.write(&[filename.len() as u8])?;
                    stream.write(filename.as_bytes())?;
                },
                "save" => {
                    if t+1 >= tokens.len() {
                        stream.write(&[2])?;
                    } else {
                        let filename = tokens[t+1];
                        stream.write(&[1])?;
                        stream.write(&[filename.len() as u8])?;
                        stream.write(filename.as_bytes())?;
                    }
                },
                "show" => {
                    stream.write(&[4])?;
                    let mut length = [0; 8];
                    stream.read(&mut length)?;
                    let length = u64::from_be_bytes(length);
                    let mut printout = Vec::new();
                    printout.resize(length as usize, 0);
                    stream.read(&mut printout[0..length as usize])?;
                    let printout = String::from_utf8(printout.to_vec()).unwrap();
                    println!("{}", printout);
                },
                "add" => {
                    let add_type = tokens[t+1];
                    match add_type {
                        "node" => {
                            let node = tokens[t+2];
                            stream.write(&[5])?;
                            stream.write(&(node.len() as u64).to_be_bytes())?;
                            stream.write(node.to_bytes())?;
                        },
                        "values" => {
                            let name = tokens[t+2];
                            let id = usize::from_str(tokens[t+3])?;
                            let val_type = tokens[t+4];
                            // TODO: make this work aaaaaaaaaaa
                            stream.write(&[6])?;
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
    Ok(())
}
