use std::collections::HashMap;
use std::collections::VecDeque;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use std::env::args;
use std::process::exit;
use std::convert::TryInto;
use std::io::{Write, Read};
use std::sync::{Arc, RwLock};

// TODO:
//   1. See line ~103
//   2. Serializing and Deserializing
//   3. Graph operations
//   4. Querying

struct Graph {
    graph: HashMap<u64, Node>,
    counter: u64
}
// Methods for making and mutating the graph
impl Graph {
    fn new() -> Self {
	return Self {
	    graph: HashMap::new(),
	    counter: 0
	}
    }
}
// Methods concerning serialization
impl Graph {
    fn to_bytes(&self) -> Vec<u8> {
	let mut out: Vec<u8> = Vec::new();
	for (_, n) in self.graph.iter() {
	    out.append(&mut n.to_bytes());
	}
	return out
    }
    fn from_bytes(bytes: Vec<u8>) -> Self {
	let mut out: Graph = Graph::new();
	for i in bytes.split(|byte| *byte == 0x0a) {
	    if !i.is_empty() {
		let node = Node::from_bytes(i.to_vec());
		out.graph.insert(node.id, node);
	    }
	}
	return out
    }
}
impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	for (_, n) in self.graph.iter() {
	    write!(f, "\n------\n{}\n------", n)?;
	}
	Ok(())
    }
}

struct Node {
    name: String,
    id: u64,
    value: Val,
    // set of ids for the interlinked nodes
    relations: Vec<u64>
}
// Methods concerning serialization
impl Node {
    // a node is encoded in the following format:
    // id,name,value,relations\n
    // example:
    // 0,Node1,0x00 128,1:2:\n
    // which is read as
    // node with id 0, name 'Node1', holding a number value of 128, and linked to nodes 1 and 2
    fn to_bytes(&self) -> Vec<u8> {
	const COMMA: u8 = 0x2c; // this is the separator ','
	const COLON: u8 = 0x4a; // this is the separator ':' used for relations
	let mut out: Vec<u8> = Vec::new();
	out.append(&mut self.id.to_be_bytes().to_vec());
	out.push(COMMA);
	out.append(&mut self.name.as_bytes().to_vec());
	out.push(COMMA);
	out.append(&mut self.value.to_bytes());
	out.push(COMMA);
	for i in self.relations.iter() {
	    out.append(&mut i.to_be_bytes().to_vec());
	    out.push(COLON);
	}
	out.push(0x0a); // this is a newline, at least for unix systems
	return out
    }
    fn from_bytes(bytes: Vec<u8>) -> Self {
	const COMMA: u8 = 0x2c; // this is the separator ','
	const COLON: u8 = 0x4a; // this is the separator ':' used for relations
	let mut fields = bytes.split(|byte| *byte == COMMA);
	let id = u64::from_be_bytes(fields
				    .next()
				    .expect("Could not get id field from byte array!")
				    .try_into()
				    .expect("Could not convert id byte slice to array!"));
	let name = String::from_utf8(fields
				     .next()
				     .expect("Could not get name field from byte array!")
				     .to_vec())
	    .expect("Could not convert byte array to string!");
	let value = Val::from_bytes(fields
				    .next()
				    .expect("Could not get value field from byte array!")
				    .to_vec());
	let mut relations: Vec<u64> = Vec::new();
	for i in fields.next()
	    .expect("Could not get relations field from byte array!")
	    .split(|byte| *byte == COLON) {
		if !i.is_empty() {
		    relations.push(
			u64::from_be_bytes(i.try_into()
					   .expect("Could not convert relations byte slice to array!"))
		    );
		}
	    }
	return Node {id: id, name: name, value: value, relations: relations}
    }
}
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "id:    {}\nname:  {}\nvalue: {}\nlinks: {:?}",
	           self.id, self.name, self.value, self.relations)
    }
}

enum Val {
    None,
    Num(isize),
    Txt(String),
    Bool(bool),
}
// Methods concerning serialization
impl Val {
    fn to_bytes(&self) -> Vec<u8> {
	let mut out: Vec<u8> = Vec::new();
	match self {
	    Self::None => {
		out.push(0);
	    },
	    Self::Num(x) => {
		out.push(1);
		out.append(&mut x.to_be_bytes().to_vec());
	    },
	    Self::Txt(s) => {
		out.push(2);
		out.append(&mut s.as_bytes().to_vec());
	    },
	    Self::Bool(b) => {
		out.push(3);
		if *b { out.push(1); } else { out.push(0) }
	    }
	}
	return out
    }
    fn from_bytes(bytes: Vec<u8>) -> Self {
	let type_sig = bytes[0];
	let val_bytes = &bytes[1..];
	match type_sig {
	    0 => return Self::None,
	    1 => return Self::Num(isize::from_be_bytes(val_bytes
						       .try_into()
						       .expect("Could not convert value byte slice to array!"))),
	    2 => return Self::Txt(String::from_utf8(val_bytes.to_vec())
				  .expect("Could not convert value byte array to string!")),
	    3 => return Self::Bool(if val_bytes[0] == 1 {true} else {false}),
	    _ => unreachable!()
	}
    }
}
impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	match self {
	    Self::None    => write!(f, "None"),
	    Self::Num(x)  => write!(f, "{}", x),
	    Self::Txt(s)  => write!(f, "{}", s),
	    Self::Bool(b) => write!(f, "{}", b)
	}
    }
}

trait Wrappable           {fn wrap(self) -> Val;}
impl Wrappable for isize  {fn wrap(self) -> Val {Val::Num(self)}}
impl Wrappable for String {fn wrap(self) -> Val {Val::Txt(self)}}
impl Wrappable for &str   {fn wrap(self) -> Val {Val::Txt(self.to_string())}}
impl Wrappable for bool   {fn wrap(self) -> Val {Val::Bool(self)}}
impl Wrappable for ()     {fn wrap(self) -> Val {Val::None}}

fn main() {
    let mut args = args().collect::<VecDeque<String>>();
    // can't use 69 cuz that requires root privs :cope:
    let mut ip = "127.0.0.1:1984".to_string();
    let mut open_filename = String::new();
    // remove exec path from argv
    args.pop_front();
    // get args 
    while args.len() > 0 {
	let current = &args.pop_front().expect("Couldn't pop from argv!")[..];
	match current {
	    "-i" => {
		if let Some(tmp) = args.pop_front() {
		    ip = tmp;
		} else {
		    println!("No value for -i given!");
		    exit(0);
		}
	    },
	    "-o" => {
		if let Some(tmp) = args.pop_front() {
		    open_filename = tmp;
		} else {
		    println!("No value for -o given!");
		    exit(0);
		}
	    },
	    t => {
		println!("Unidentified token [{}]!", t);
		exit(0);
	    }
	}
    }
    // open listener
    println!("Opening TcpListener on {}", ip);
    let listener;
    match TcpListener::bind(ip) {
	Ok(tmp) => {
	    listener = tmp;
	},
	Err(e) => {
	    println!("Could not establish TcpListener: {}", e);
	    exit(0);
	},
    }
    // if -o is provided, open given file, else create new graph
    let graph;
    if !open_filename.is_empty() {
	graph = Arc::new(RwLock::new(Graph::from_bytes(std::fs::read(open_filename).expect("Couldn't open file!"))));
    } else {
	graph = Arc::new(RwLock::new(Graph::new()));
    }
    // look through incoming connections and spawn new threads for each
    // TODO: some way to shut down daze without C-c
    for stream in listener.incoming() {
	match stream {
	    Ok(s) => {
		// spawn new instance of RwLock to pass to the thread
		let mut graph_copy = Arc::clone(&graph);
		// spawn new thread which handles requests
		thread::spawn(move || {handle_requests(s, graph_copy)});
	    },
	    Err(e) => {
		println!("Could not establish connection: {}", e);
		println!("Continuing regardless!");
	    },
	}
    }
}

// TODO: Do the actual request handling
fn handle_requests(mut stream: TcpStream, mut graph: Arc<RwLock<Graph>>) -> std::io::Result<()> {
    let stream_address = stream.peer_addr()?;
    println!("accepted a stream: {:?}", stream_address);
    loop {
	let mut command = [0; 1];
	stream.read(&mut command)?;
	match command[0] {
	    0 => { // shutdown
		println!("stream has shutdown: {}", stream_address);
		break
	    },
	    1 => { // print
		// Tries to get read access, if it takes longer sends message
		// Simply panics if the RwLock is poisoned
		let first_attempt = graph.try_read();
		if first_attempt.is_err() {
		    println!("The graph is currently inacessible, waiting to receive read access!");
		    let snapshot = graph.read().unwrap();
		    println!("{}", snapshot);
		} else {
		    println!("{}", first_attempt.unwrap());
		}
	    },
	    2 => { // open file 
		let mut name_length_packet = [0;8];
		stream.read(&mut name_length_packet)?;
		let name_length = u64::from_be_bytes(name_length_packet);
		let mut name_packet: Vec<u8> = Vec::new();
		name_packet.resize(name_length as usize, 0);
		stream.read(&mut name_packet[0..name_length as usize])?;
		let name = String::from_utf8(name_packet).expect("Couldn't convert name packet to string!");
		// TODO: decide how to handle a thread opening a file
		// iows: should it be opened locally or globally
	    },
	    3 => { // write file
		// get filename
		let mut name_length_packet = [0;8];
		stream.read(&mut name_length_packet)?;
		let name_length = u64::from_be_bytes(name_length_packet);
		let mut name_packet: Vec<u8> = Vec::new();
		name_packet.resize(name_length as usize, 0);
		stream.read(&mut name_packet[0..name_length as usize])?;
		let name = String::from_utf8(name_packet).expect("Couldn't convert name packet to string!");
		// get graph
		let first_attempt = graph.try_read();
		let snapshot;
		if first_attempt.is_err() {
		    println!("The graph is currently inacessible, waiting to receive read access!");
		    snapshot = graph.read().unwrap();
		} else {
		    snapshot = first_attempt.unwrap();
		}
		std::fs::write(name, snapshot.to_bytes());
	    },
	    _ => unreachable!()
	}
    }
    Ok(())
}
