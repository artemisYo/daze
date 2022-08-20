use std::collections::HashMap;
use std::collections::VecDeque;
use std::convert::TryInto;
use std::env::args;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::sync::{Arc, RwLock};
use std::thread;

// TODO:
//   1. Query tcp calls
//   2. Also that thing with opening a file

#[derive(Debug)]
struct Graph {
    graph: HashMap<u64, Node>,
    name_lookup: HashMap<String, Vec<u64>>,
    value_lookup: HashMap<Val, Vec<u64>>,
    counter: u64,
}
enum Orders {
    First,
    Last,
}
// Methods concerning querying
impl Graph {
    fn select_by_name(&self, name: &str) -> Vec<&Node> {
        let mut out: Vec<&Node> = Vec::new();
        if let Some(indices) = self.name_lookup.get(name) {
            for i in indices.iter() {
                out.push(self.graph.get(i).unwrap());
            }
        }
        return out;
    }
    fn select_amount_by_name(&self, name: &str, mut amount: usize, ordering: Orders) -> Vec<&Node> {
        let mut out: Vec<&Node> = Vec::new();
        if let Some(indices) = self.name_lookup.get(name) {
            let mut indices = indices.clone();
            match ordering {
                Orders::First => indices.sort_unstable_by(|a, b| a.cmp(b)),
                Orders::Last => indices.sort_unstable_by(|a, b| b.cmp(a)),
            }
            if amount >= indices.len() {
                amount = indices.len() - 1
            }
            for i in indices[0..amount + 1].iter() {
                out.push(self.graph.get(i).unwrap());
            }
        }
        return out;
    }
    fn select_by_value<T: Wrappable>(&self, value: T) -> Vec<&Node> {
        let mut out: Vec<&Node> = Vec::new();
        if let Some(indices) = self.value_lookup.get(&value.wrap()) {
            for i in indices.iter() {
                out.push(self.graph.get(i).unwrap());
            }
        }
        return out;
    }
    fn select_amount_by_value<T: Wrappable>(
        &self,
        value: T,
        mut amount: usize,
        ordering: Orders,
    ) -> Vec<&Node> {
        let mut out: Vec<&Node> = Vec::new();
        if let Some(indices) = self.value_lookup.get(&value.wrap()) {
            let mut indices = indices.clone();
            match ordering {
                Orders::First => indices.sort_unstable_by(|a, b| a.cmp(b)),
                Orders::Last => indices.sort_unstable_by(|a, b| b.cmp(a)),
            }
            if amount >= indices.len() {
                amount = indices.len() - 1
            }
            for i in indices[0..amount + 1].iter() {
                out.push(self.graph.get(i).unwrap());
            }
        }
        return out;
    }
    fn select_backlinks(&self, id: u64) -> Vec<&Node> {
        let mut out: Vec<&Node> = Vec::new();
        if let Some(n) = self.graph.get(&id) {
            let indices = &n.backlinks;
            for i in indices.iter() {
                out.push(self.graph.get(i).unwrap());
            }
        }
        return out;
    }
    fn select_amount_backlinks(&self, id: u64, mut amount: usize, ordering: Orders) -> Vec<&Node> {
        let mut out: Vec<&Node> = Vec::new();
        if let Some(n) = self.graph.get(&id) {
            let indices = &n.backlinks;
            for i in indices.iter() {
                out.push(self.graph.get(i).unwrap());
            }
        }
        match ordering {
            Orders::First => out.sort_unstable_by(|a, b| a.id.partial_cmp(&b.id).unwrap()),
            Orders::Last => out.sort_unstable_by(|a, b| b.id.partial_cmp(&a.id).unwrap()),
        }
        if amount >= out.len() {
            amount = out.len() - 1;
        }
        return out[0..amount + 1].to_vec();
    }
    fn select_link(&self, id: u64) -> Vec<&Node> {
        let mut out: Vec<&Node> = Vec::new();
        if let Some(n) = self.graph.get(&id) {
            let indices = &n.relations;
            for i in indices.iter() {
                out.push(self.graph.get(i).unwrap());
            }
        }
        return out;
    }
    fn select_amount_link(&self, id: u64, mut amount: usize, ordering: Orders) -> Vec<&Node> {
        let mut out: Vec<&Node> = Vec::new();
        if let Some(n) = self.graph.get(&id) {
            let indices = &n.relations;
            for i in indices.iter() {
                out.push(self.graph.get(i).unwrap());
            }
        }
        match ordering {
            Orders::First => out.sort_unstable_by(|a, b| a.id.partial_cmp(&b.id).unwrap()),
            Orders::Last => out.sort_unstable_by(|a, b| b.id.partial_cmp(&a.id).unwrap()),
        }
        if amount >= out.len() {
            amount = out.len() - 1
        }
        return out[0..amount + 1].to_vec();
    }
    fn select_by_id(&self, id: u64) -> Option<&Node> {
        self.graph.get(&id)
    }
}
// Methods for making and mutating the graph
impl Graph {
    fn new() -> Self {
        return Self {
            graph: HashMap::new(),
            name_lookup: HashMap::new(),
            value_lookup: HashMap::new(),
            counter: 0,
        };
    }
    fn append<T: Wrappable>(&mut self, name: &str, value: T) -> u64 {
        let node = Node {
            name: name.to_string(),
            id: self.counter,
            value: value.wrap(),
            relations: Vec::new(),
            backlinks: Vec::new(),
        };
        if let Some(mut res) = self.name_lookup.insert(node.name.clone(), vec![node.id]) {
            let n = self.name_lookup.get_mut(&node.name).unwrap();
            n.append(&mut res);
        }
        if let Some(mut res) = self.value_lookup.insert(node.value.clone(), vec![node.id]) {
            let v = self.value_lookup.get_mut(&node.value).unwrap();
            v.append(&mut res);
        }
        self.graph.insert(self.counter, node);
        self.counter += 1;
        return self.counter - 1;
    }
    fn insert<T: Wrappable>(&mut self, name: &str, value: T, id: u64) {
        let node = Node {
            name: name.to_string(),
            id: id,
            value: value.wrap(),
            relations: Vec::new(),
            backlinks: Vec::new(),
        };
        if let Some(mut res) = self.name_lookup.insert(node.name.clone(), vec![node.id]) {
            let n = self.name_lookup.get_mut(&node.name).unwrap();
            n.append(&mut res);
        }
        if let Some(mut res) = self.value_lookup.insert(node.value.clone(), vec![node.id]) {
            let v = self.value_lookup.get_mut(&node.value).unwrap();
            v.append(&mut res);
        }
        self.graph.insert(id, node);
        if self.counter < id {
            self.counter = id
        }
    }
    fn delete(&mut self, id: u64) -> Option<Node> {
        if let Some(node) = self.graph.remove(&id) {
            for i in node.relations.iter() {
                let n = self.graph.get_mut(i).unwrap();
                n.backlinks.retain(|id| *id != node.id);
            }
            self.name_lookup.remove(&node.name);
            self.value_lookup.remove(&node.value);
            Some(node)
        } else {
            None
        }
    }
    fn set_relations(&mut self, id: u64, relations: Vec<u64>) -> Result<(), String> {
        let old_rels: Vec<u64>;
        if let Some(node) = self.graph.get_mut(&id) {
            old_rels = node.relations.clone();
            node.relations = relations;
        } else {
            return Err("Could not find node under given id!".to_string());
        }
        for i in old_rels.iter() {
            let n = self.graph.get_mut(i).unwrap();
            n.backlinks.retain(|idk| *idk != id);
        }
        let node = self.graph.get(&id).unwrap().clone();
        for i in node.relations.iter() {
            if let Some(n) = self.graph.get_mut(i) {
                n.backlinks.push(id);
            } else {
                return Err(format!(
                    "Could not find node under id in relations: [{}]",
                    i
                ));
            }
        }
        Ok(())
    }
    fn append_relations(&mut self, id: u64, mut relations: Vec<u64>) -> Result<(), String> {
        if let Some(node) = self.graph.get_mut(&id) {
            node.relations.append(&mut relations);
        } else {
            return Err("Could not find node under given id!".to_string());
        }
        for i in relations.iter() {
            if let Some(node) = self.graph.get_mut(i) {
                node.backlinks.push(id);
            } else {
                return Err(format!(
                    "Could not find node under id in relations: [{}]",
                    i
                ));
            }
        }
        Ok(())
    }
    fn set_name(&mut self, id: u64, name: &str) -> Result<(), String> {
        if let Some(node) = self.graph.get_mut(&id) {
            self.name_lookup.remove(&node.name);
            node.name = name.to_string();
            if let Some(mut res) = self.name_lookup.insert(node.name.clone(), vec![node.id]) {
                let n = self.name_lookup.get_mut(&node.name).unwrap();
                n.append(&mut res);
            }
        } else {
            return Err("Could not find node under given id!".to_string());
        }
        Ok(())
    }
    fn set_value<T: Wrappable>(&mut self, id: u64, value: T) -> Result<(), String> {
        if let Some(node) = self.graph.get_mut(&id) {
            self.value_lookup.remove(&node.value);
            node.value = value.wrap();
            if let Some(mut res) = self.value_lookup.insert(node.value.clone(), vec![node.id]) {
                let n = self.value_lookup.get_mut(&node.value).unwrap();
                n.append(&mut res);
            }
        } else {
            return Err("Could not find node under given id!".to_string());
        }
        Ok(())
    }
    fn print_graph(&self) -> String {
        let mut out = String::new();
        out.push_str("digraph {\n");
        for (_, v) in self.graph.iter() {
            out.push_str(&format!("{}[label=\"{}\"]\n", v.id, v));
            if !v.relations.is_empty() {
                for i in v.relations.iter() {
                    out.push_str(&format!("{} -> {}\n", v.id, i));
                }
            }
        }
        out.push_str("}");
        return out;
    }
}
// Methods concerning serialization
impl Graph {
    fn to_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        for (_, n) in self.graph.iter() {
            out.append(&mut n.to_bytes());
        }
        return out;
    }
    fn from_bytes(bytes: Vec<u8>) -> Self {
        let mut out: Graph = Graph::new();
        for i in bytes.split(|byte| *byte == 0x0a) {
            if !i.is_empty() {
                let node = Node::from_bytes(i.to_vec());
                if let Some(n) = out.name_lookup.get_mut(&node.name) {
                    n.push(node.id);
                } else {
                    out.name_lookup.insert(node.name.clone(), vec![node.id]);
                }
                if let Some(n) = out.value_lookup.get_mut(&node.value) {
                    n.push(node.id);
                } else {
                    out.value_lookup.insert(node.value.clone(), vec![node.id]);
                }
                out.graph.insert(node.id, node);
            }
        }
        for (k, v) in out.graph.clone().iter() {
            for i in v.relations.iter() {
                let n = out.graph.get_mut(i).unwrap();
                n.backlinks.push(*k);
            }
        }
        return out;
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

#[derive(Debug, Clone)]
struct Node {
    name: String,
    id: u64,
    value: Val,
    // set of ids for the interlinked nodes
    relations: Vec<u64>,
    backlinks: Vec<u64>,
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
        return out;
    }
    fn from_bytes(bytes: Vec<u8>) -> Self {
        const COMMA: u8 = 0x2c; // this is the separator ','
        const COLON: u8 = 0x4a; // this is the separator ':' used for relations
        let mut fields = bytes.split(|byte| *byte == COMMA);
        let id = u64::from_be_bytes(
            fields
                .next()
                .expect("Could not get id field from byte array!")
                .try_into()
                .expect("Could not convert id byte slice to array!"),
        );
        let name = String::from_utf8(
            fields
                .next()
                .expect("Could not get name field from byte array!")
                .to_vec(),
        )
        .expect("Could not convert byte array to string!");
        let value = Val::from_bytes(
            fields
                .next()
                .expect("Could not get value field from byte array!")
                .to_vec(),
        );
        let mut relations: Vec<u64> = Vec::new();
        for i in fields
            .next()
            .expect("Could not get relations field from byte array!")
            .split(|byte| *byte == COLON)
        {
            if !i.is_empty() {
                relations.push(u64::from_be_bytes(
                    i.try_into()
                        .expect("Could not convert relations byte slice to array!"),
                ));
            }
        }
        return Node {
            id: id,
            name: name,
            value: value,
            relations: relations,
            backlinks: Vec::new(),
        };
    }
}
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id:    {}\nname:  {}\nvalue: {}\nlinks: {:?}",
            self.id, self.name, self.value, self.relations
        )
    }
}

#[derive(Clone, Debug)]
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
            }
            Self::Num(x) => {
                out.push(1);
                out.append(&mut x.to_be_bytes().to_vec());
            }
            Self::Txt(s) => {
                out.push(2);
                out.append(&mut s.as_bytes().to_vec());
            }
            Self::Bool(b) => {
                out.push(3);
                if *b {
                    out.push(1);
                } else {
                    out.push(0)
                }
            }
        }
        return out;
    }
    fn from_bytes(bytes: Vec<u8>) -> Self {
        let type_sig = bytes[0];
        let val_bytes = &bytes[1..];
        match type_sig {
            0 => return Self::None,
            1 => {
                return Self::Num(isize::from_be_bytes(
                    val_bytes
                        .try_into()
                        .expect("Could not convert value byte slice to array!"),
                ))
            }
            2 => {
                return Self::Txt(
                    String::from_utf8(val_bytes.to_vec())
                        .expect("Could not convert value byte array to string!"),
                )
            }
            3 => return Self::Bool(if val_bytes[0] == 1 { true } else { false }),
            _ => unreachable!(),
        }
    }
}
impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Num(x) => write!(f, "{}", x),
            Self::Txt(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b),
        }
    }
}
impl std::cmp::PartialEq for Val {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::None => match other {
                Self::None => return true,
                _ => return false,
            },
            Self::Num(x) => match other {
                Self::Num(y) => return x == y,
                _ => return false,
            },
            Self::Txt(s) => match other {
                Self::Txt(t) => return s == t,
                _ => return false,
            },
            Self::Bool(b) => match other {
                Self::Bool(c) => return b == c,
                _ => return false,
            },
        }
    }
}
impl std::cmp::Eq for Val {}
impl std::hash::Hash for Val {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::None => 0.hash(state),
            Self::Num(x) => {
                1.hash(state);
                x.hash(state)
            }
            Self::Txt(s) => {
                2.hash(state);
                s.hash(state)
            }
            Self::Bool(b) => {
                3.hash(state);
                b.hash(state)
            }
        }
    }
}

trait Wrappable {
    fn wrap(self) -> Val;
}
impl Wrappable for isize {
    fn wrap(self) -> Val {
        Val::Num(self)
    }
}
impl Wrappable for String {
    fn wrap(self) -> Val {
        Val::Txt(self)
    }
}
impl Wrappable for &str {
    fn wrap(self) -> Val {
        Val::Txt(self.to_string())
    }
}
impl Wrappable for bool {
    fn wrap(self) -> Val {
        Val::Bool(self)
    }
}
impl Wrappable for () {
    fn wrap(self) -> Val {
        Val::None
    }
}
impl Wrappable for Val {
    fn wrap(self) -> Val {
        self
    }
}

trait QueryingAriths {
    fn union(&self, _: &Self) -> Self; // OR
    fn intersection(&self, _: &Self) -> Self; // AND
    fn difference(&self, _: &Self) -> Self; // XOR
}
// Unions and shit
impl<T> QueryingAriths for Vec<T>
where
    T: PartialEq + Clone,
{
    fn union(&self, other: &Self) -> Self {
        let mut out: Self = Vec::new();
        for i in self.iter() {
            out.push(i.clone());
        }
        for i in other.iter() {
            if !out.contains(i) {
                out.push(i.clone());
            }
        }
        return out;
    }
    fn intersection(&self, other: &Self) -> Self {
        let mut out: Self = Vec::new();
        for i in self.iter() {
            if other.contains(i) {
                out.push(i.clone());
            }
        }
        return out;
    }
    fn difference(&self, other: &Self) -> Self {
        let mut out: Self = Vec::new();
        for i in self.iter() {
            if !other.contains(i) {
                out.push(i.clone());
            }
        }
        for i in other.iter() {
            if !self.contains(i) {
                out.push(i.clone());
            }
        }
        return out;
    }
}

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
            }
            "-o" => {
                if let Some(tmp) = args.pop_front() {
                    open_filename = tmp;
                } else {
                    println!("No value for -o given!");
                    exit(0);
                }
            }
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
        }
        Err(e) => {
            println!("Could not establish TcpListener: {}", e);
            exit(0);
        }
    }
    // if -o is provided, open given file, else create new graph
    let graph;
    if !open_filename.is_empty() {
        graph = Arc::new(RwLock::new(Graph::from_bytes(
            std::fs::read(open_filename).expect("Couldn't open file!"),
        )));
    } else {
        graph = Arc::new(RwLock::new(Graph::new()));
    }
    // look through incoming connections and spawn new threads for each
    // TODO: some way to shut down daze without C-c
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                // spawn new instance of RwLock to pass to the thread
                let graph_copy = Arc::clone(&graph);
                // spawn new thread which handles requests
                thread::spawn(move || handle_requests(s, graph_copy));
            }
            Err(e) => {
                println!("Could not establish connection: {}", e);
                println!("Continuing regardless!");
            }
        }
    }
}

// TODO: Do the actual request handling
fn handle_requests(mut stream: TcpStream, graph: Arc<RwLock<Graph>>) -> std::io::Result<()> {
    let stream_address = stream.peer_addr()?;
    println!("accepted a stream: {:?}", stream_address);
    loop {
        let mut command = [0; 1];
        stream.read(&mut command)?;
        match command[0] {
            0 => {
                // shutdown
                println!("stream has shutdown: {}", stream_address);
                break;
            }
            1 => {
                // print
                println!("Entered print command!");
                // Tries to get read access, if it takes longer sends message
                // Simply panics if the RwLock is poisoned
                println!(
                    "Connection [{}] is trying to get read access!",
                    stream_address
                );
                let first_attempt = graph.try_read();
                if first_attempt.is_err() {
                    stream.write(&(67 as u64).to_be_bytes())?;
                    stream.write(
                        "The graph is currently inacessible, waiting to receive read access!"
                            .as_bytes(),
                    )?;
                    let snapshot = graph.read().unwrap();
                    let printout = format!("{}", snapshot);
                    stream.write(&(printout.len() as u64).to_be_bytes())?;
                    println!("{:?}", snapshot);
                    stream.write(printout.as_bytes())?;
                } else {
                    let snapshot = first_attempt.unwrap();
                    let printout = format!("{}", snapshot);
                    println!("{:?}", snapshot);
                    stream.write(&(printout.len() as u64).to_be_bytes())?;
                    stream.write(printout.as_bytes())?;
                }
            }
            2 => {
                // open file TODO!
                let mut name_length_packet = [0; 8];
                stream.read(&mut name_length_packet)?;
                let name_length = u64::from_be_bytes(name_length_packet);
                let mut name_packet: Vec<u8> = Vec::new();
                name_packet.resize(name_length as usize, 0);
                stream.read(&mut name_packet[0..name_length as usize])?;
                let _name = String::from_utf8(name_packet)
                    .expect("Couldn't convert name packet to string!");
                // TODO: decide how to handle a thread opening a file
                // iows: should it be opened locally or globally
            }
            3 => {
                // write file
                println!("Entered write file command!");
                // get filename
                let mut name_length_packet = [0; 8];
                stream.read(&mut name_length_packet)?;
                let name_length = u64::from_be_bytes(name_length_packet);
                let mut name_packet: Vec<u8> = Vec::new();
                name_packet.resize(name_length as usize, 0);
                stream.read(&mut name_packet[0..name_length as usize])?;
                let name = String::from_utf8(name_packet)
                    .expect("Couldn't convert name packet to string!");
                println!("Name read!");
                // get graph
                println!(
                    "Connection [{}] is trying to get read access!",
                    stream_address
                );
                let first_attempt = graph.try_read();
                let snapshot;
                if first_attempt.is_err() {
                    stream.write(&(67 as u64).to_be_bytes())?;
                    stream.write(
                        "The graph is currently inacessible, waiting to receive read access!"
                            .as_bytes(),
                    )?;
                    snapshot = graph.read().unwrap();
                } else {
                    snapshot = first_attempt.unwrap();
                }
                std::fs::write(&name, snapshot.to_bytes())?;
                let printout = format!("Wrote file: [{}]", name);
                println!("Wrote file: [{}]", name);
                stream.write(&(printout.len() as usize).to_be_bytes())?;
                stream.write(printout.as_bytes())?;
            }
            4 => {
                // append node
                println!("Entered append command!");
                // get values for node
                let mut name_length_packet = [0; 8];
                stream.read(&mut name_length_packet)?;
                let name_length = u64::from_be_bytes(name_length_packet);
                let mut name_packet: Vec<u8> = Vec::new();
                name_packet.resize(name_length as usize, 0);
                stream.read(&mut name_packet[0..name_length as usize])?;
                let name = String::from_utf8(name_packet)
                    .expect("Couldn't convert name pacjet to string!");
                println!("Name read!");
                let mut value_type = [0; 1];
                let value: Val;
                stream.read(&mut value_type)?;
                match value_type[0] {
                    0 => {
                        value = Val::None;
                    }
                    1 => {
                        let mut num_packet = [0; 8];
                        stream.read(&mut num_packet)?;
                        let num = isize::from_be_bytes(num_packet);
                        value = Val::Num(num);
                    }
                    2 => {
                        let mut txt_size_packet = [0; 8];
                        stream.read(&mut txt_size_packet)?;
                        let txt_size = u64::from_be_bytes(txt_size_packet);
                        let mut txt_packet: Vec<u8> = Vec::new();
                        txt_packet.resize(txt_size as usize, 0);
                        stream.read(&mut txt_packet[0..txt_size as usize])?;
                        let txt = String::from_utf8(txt_packet)
                            .expect("Could not convert text packet to string!");
                        value = Val::Txt(txt);
                    }
                    3 => {
                        let mut bool_packet = [0; 1];
                        stream.read(&mut bool_packet)?;
                        match bool_packet[0] {
                            0 => value = Val::Bool(false),
                            1 => value = Val::Bool(true),
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
                println!("Value read!");
                // get graph
                println!(
                    "Connection [{}] is trying to get read access!",
                    stream_address
                );
                let first_attempt = graph.try_write();
                let mut snapshot;
                if first_attempt.is_err() {
                    stream.write(&(67 as u64).to_be_bytes())?;
                    stream.write(
                        "The graph is currently inacessible, waiting to receive read access!"
                            .as_bytes(),
                    )?;
                    snapshot = graph.write().unwrap();
                } else {
                    snapshot = first_attempt.unwrap();
                }
                // do the thing
                let aids = snapshot.append(&name, value);
                println!("Appended id: [{}]", aids);
                stream.write(&aids.to_be_bytes())?;
            }
            5 => {
                // insert node
                println!("Entered insert command!");
                // get values for node
                let mut name_length_packet = [0; 8];
                stream.read(&mut name_length_packet)?;
                let name_length = u64::from_be_bytes(name_length_packet);
                let mut name_packet: Vec<u8> = Vec::new();
                name_packet.resize(name_length as usize, 0);
                stream.read(&mut name_packet[0..name_length as usize])?;
                let name = String::from_utf8(name_packet)
                    .expect("Couldn't convert name pacjet to string!");
                println!("Name read!");
                let mut value_type = [0; 1];
                let value: Val;
                stream.read(&mut value_type)?;
                match value_type[0] {
                    0 => {
                        value = Val::None;
                    }
                    1 => {
                        let mut num_packet = [0; 8];
                        stream.read(&mut num_packet)?;
                        let num = isize::from_be_bytes(num_packet);
                        value = Val::Num(num);
                    }
                    2 => {
                        let mut txt_size_packet = [0; 8];
                        stream.read(&mut txt_size_packet)?;
                        let txt_size = u64::from_be_bytes(txt_size_packet);
                        let mut txt_packet: Vec<u8> = Vec::new();
                        txt_packet.resize(txt_size as usize, 0);
                        stream.read(&mut txt_packet[0..txt_size as usize])?;
                        let txt = String::from_utf8(txt_packet)
                            .expect("Could not convert text packet to string!");
                        value = Val::Txt(txt);
                    }
                    3 => {
                        let mut bool_packet = [0; 1];
                        stream.read(&mut bool_packet)?;
                        match bool_packet[0] {
                            0 => value = Val::Bool(false),
                            1 => value = Val::Bool(true),
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
                println!("Value read!");
                let mut id_packet = [0; 8];
                stream.read(&mut id_packet)?;
                let id = u64::from_be_bytes(id_packet);
                println!("Id read!");
                // get graph
                println!(
                    "Connection [{}] is trying to get read access!",
                    stream_address
                );
                let first_attempt = graph.try_write();
                let mut snapshot;
                if first_attempt.is_err() {
                    stream.write(&(67 as u64).to_be_bytes())?;
                    stream.write(
                        "The graph is currently inacessible, waiting to receive read access!"
                            .as_bytes(),
                    )?;
                    snapshot = graph.write().unwrap();
                } else {
                    snapshot = first_attempt.unwrap();
                }
                // do the thing
                snapshot.insert(&name, value, id);
            }
            6 => {
                // delete node
                println!("Entered delete command!");
                // get id
                let mut id_packet = [0; 8];
                stream.read(&mut id_packet)?;
                let id = u64::from_be_bytes(id_packet);
                println!("Read id!");
                // get graph
                println!(
                    "Connection [{}] is trying to get read access!",
                    stream_address
                );
                let mut snapshot;
                let first_attempt = graph.try_write();
                if first_attempt.is_err() {
                    stream.write(&(67 as u64).to_be_bytes())?;
                    stream.write(
                        "The graph is currently inacessible, waiting to receive read access!"
                            .as_bytes(),
                    )?;
                    snapshot = graph.write().unwrap();
                } else {
                    snapshot = first_attempt.unwrap();
                }
                // do the thing
                if let Some(_) = snapshot.delete(id) { /*dkals*/ }
            }
            7 => {
                // set relations for node
                println!("Entered set relations command!");
                let mut id_packet = [0; 8];
                stream.read(&mut id_packet)?;
                let id = u64::from_be_bytes(id_packet);
                println!("Read id!");
                let mut relations: Vec<u64> = Vec::new();
                let mut relations_number_packet = [0; 8];
                stream.read(&mut relations_number_packet)?;
                let relations_number = u64::from_be_bytes(relations_number_packet);
                for _ in 0..relations_number {
                    let mut relation_packet = [0; 8];
                    stream.read(&mut relation_packet)?;
                    relations.push(u64::from_be_bytes(relation_packet));
                }
                println!("Read relations!");
                // get graph
                println!(
                    "Connection [{}] is trying to get read access!",
                    stream_address
                );
                let mut snapshot;
                let first_attempt = graph.try_write();
                if first_attempt.is_err() {
                    stream.write(&(67 as u64).to_be_bytes())?;
                    stream.write(
                        "The graph is currently inacessible, waiting to receive read access!"
                            .as_bytes(),
                    )?;
                    snapshot = graph.write().unwrap();
                } else {
                    snapshot = first_attempt.unwrap();
                }
                // do the thing
                if let Ok(_) = snapshot.set_relations(id, relations) { /*aaaaaa*/ }
            }
            8 => {
                // set name for node
                println!("Entered set name command!");
                let mut id_packet = [0; 8];
                stream.read(&mut id_packet)?;
                let id = u64::from_be_bytes(id_packet);
                println!("Read id!");
                let mut name_length_packet = [0; 8];
                stream.read(&mut name_length_packet)?;
                let name_length = u64::from_be_bytes(name_length_packet);
                let mut name_packet: Vec<u8> = Vec::new();
                name_packet.resize(name_length as usize, 0);
                stream.read(&mut name_packet)?;
                let name = String::from_utf8(name_packet)
                    .expect("Couldn't convert name packet to string!");
                println!("Read name!");
                // get graph
                println!(
                    "Connection [{}] is trying to get read access!",
                    stream_address
                );
                let mut snapshot;
                let first_attempt = graph.try_write();
                if first_attempt.is_err() {
                    stream.write(&(67 as u64).to_be_bytes())?;
                    stream.write(
                        "The graph is currently inacessible, waiting to receive read access!"
                            .as_bytes(),
                    )?;
                    snapshot = graph.write().unwrap();
                } else {
                    snapshot = first_attempt.unwrap();
                }
                // do the thing
                if let Ok(_) = snapshot.set_name(id, &name) { /*idk*/ }
            }
            9 => {
                // set value for node
                println!("Entered set value command!");
                let mut id_packet = [0; 8];
                stream.read(&mut id_packet)?;
                let id = u64::from_be_bytes(id_packet);
                println!("Read id!");
                // get value
                let mut value_type = [0; 1];
                let value: Val;
                stream.read(&mut value_type)?;
                match value_type[0] {
                    0 => {
                        value = Val::None;
                    }
                    1 => {
                        let mut num_packet = [0; 8];
                        stream.read(&mut num_packet)?;
                        let num = isize::from_be_bytes(num_packet);
                        value = Val::Num(num);
                    }
                    2 => {
                        let mut txt_size_packet = [0; 8];
                        stream.read(&mut txt_size_packet)?;
                        let txt_size = u64::from_be_bytes(txt_size_packet);
                        let mut txt_packet: Vec<u8> = Vec::new();
                        txt_packet.resize(txt_size as usize, 0);
                        stream.read(&mut txt_packet[0..txt_size as usize])?;
                        let txt = String::from_utf8(txt_packet)
                            .expect("Could not convert text packet to string!");
                        value = Val::Txt(txt);
                    }
                    3 => {
                        let mut bool_packet = [0; 1];
                        stream.read(&mut bool_packet)?;
                        match bool_packet[0] {
                            0 => value = Val::Bool(false),
                            1 => value = Val::Bool(true),
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
                println!("Read value!");
                // get graph
                println!(
                    "Connection [{}] is trying to get read access!",
                    stream_address
                );
                let mut snapshot;
                let first_attempt = graph.try_write();
                if first_attempt.is_err() {
                    stream.write(&(67 as u64).to_be_bytes())?;
                    stream.write(
                        "The graph is currently inacessible, waiting to receive read access!"
                            .as_bytes(),
                    )?;
                    snapshot = graph.write().unwrap();
                } else {
                    snapshot = first_attempt.unwrap();
                }
                // do the thing
                if let Ok(_) = snapshot.set_value(id, value) { /* idk you could print that it succeeded wtf*/
                }
            }
            10 => {
                // print graph
                println!(
                    "Connection [{}] is trying to get read access!",
                    stream_address
                );
                let dotcode;
                let first_attempt = graph.try_read();
                if first_attempt.is_err() {
                    println!("The graph is currently inacessible, waiting to receive read access!");
                    let snapshot = graph.read().unwrap();
                    dotcode = snapshot.print_graph();
                } else {
                    dotcode = first_attempt.unwrap().print_graph();
                }
                println!("Attempting to write test.dot!");
                std::fs::write("test.dot", dotcode.as_bytes())?;
                println!("Attempting to execute dot -Tpng test.dot!");
                let output = std::process::Command::new("dot")
                    .arg("-Tpng")
                    .arg("test.dot")
                    .output()
                    .expect("failed executing dot!");
                println!("Attempting to write test.png!");
                std::fs::write("test.png", output.stdout)?;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}
