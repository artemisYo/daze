use std::collections::HashMap;
use std::collections::VecDeque;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use std::env::args;
use std::process::exit;
use std::io::{Write, Read};
use std::sync::{Arc, RwLock};

// TODO:
//   1. See line ~97
//   1.1 Please also see `architecture.org`
//   2. Serializing and Deserializing
//   3. Graph operations
//   4. Querying

#[derive(Debug)]
struct Graph<'a> {
    graph: HashMap<u64, Node<'a>>,
    counter: u64
}
impl<'a> Graph<'a> {
    fn new() -> Self {
	return Self {
	    graph: HashMap::new(),
	    counter: 0
	}
    }
}

#[derive(Debug)]
struct Node<'a> {
    name: &'a str,
    id: u64,
    //value: Val,
    // set of ids for the interlinked nodes
    relations: Vec<u64>
}

fn main() {
    let mut args = args().collect::<VecDeque<String>>();
    let mut ip = "127.0.0.1:69".to_string();
    let mut open_filename = "".to_string();
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
    // TODO: open file if given per -o and construct graph
    let mut graph = Arc::new(RwLock::new(Graph::new()));
    // look through incoming connections
    for stream in listener.incoming() {
	match stream {
	    Ok(s) => {
		// spawn a new reference to RwLock, to give to the thread
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
fn handle_requests(mut stream: TcpStream, mut graph_copy: Arc<RwLock<Graph>>) -> std::io::Result<()> {
    let mut stream_address = stream.peer_addr()?;
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
		// TODO: Impl display for graph and replace debug with it
		// Tries to get read access, if it takes longer sends message
		// Simply panics if the RwLock is poisoned
		let first_attempt = graph_copy.try_read();
		if first_attempt.is_err() {
		    println!("The graph is currently inacessible, waiting to receive read access!");
		    let snapshot = graph_copy.read().unwrap();
		    println!("{:?}", snapshot);
		} else {
		    println!("{:?}", first_attempt.unwrap());
		}
	    },
	    _ => unreachable!()
	}
    }
    Ok(())
}
