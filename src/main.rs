use std::fs;
use std::io::{Write, Read};
use std::io::BufWriter;
use std::collections::HashMap;
use std::net::TcpListener;

// DONE:
//   1. Serializing
//   2. Deserializing
//   3. Graph is a hashmap
//   4. Constructors
//   5. Appending nodes
//   6. Deleting nodes
//   7. Mutating nodes
//   8. Documentation

// TODO:
//   1. User Interface
//   1.1 TCP request handling
//   1.1.1 Need to add the instructions for the functions from 'DONE' 5-7
//   1.2 The UI will be it's own thing

// EH:
//   1. Querying
//   1.1 Also a format that facilitates said operations
//   2. Exporting
//   3. Think about exposing wrappable
//   4. Concurrency
//   5. Parametrize the IP to open on
//   6. Think about how to give feedback to the ui

#[derive(Debug, Clone)]
enum Val {
    None,
    Num(isize),
    Txt(String),
    Bool(bool)
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Nothing"),
            Self::Num(x) => write!(f, "{}", x),
            Self::Txt(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b)
        }
    }
}

impl Val {
    fn to_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        match self {
            Self::None    => {
                out.push(3);
            },
            Self::Num(x)  => {
                out.push(0);
                out.append(&mut Vec::from(x.to_be_bytes()));
            },
            Self::Txt(s)  => {
                out.push(1);
                out.append(&mut Vec::from(s.as_bytes()));
            },
            Self::Bool(b) => {
                out.push(2);
                if *b {
                    out.push(1);
                } else {
                    out.push(0);
                }
            },
        }
        return out
    }

    fn from_bytes(mut input: Vec<u8>) -> Self {
        let sign = input.remove(0);
        match sign {
            0 => {
                return Self::Num(isize::from_be_bytes(input.try_into().unwrap()))
            },
            1 => {
                return Self::Txt(String::from_utf8(input).unwrap())
            },
            2 => {
                if input.remove(0) == 1 {
                    return Self::Bool(true)
                } else {
                    return Self::Bool(false)
                }
            },
            3 => {
                return Self::None
            },
            _ => unreachable!()
        }
    }
}

type ID = u16;
#[derive(Debug, Clone)]
struct Node {
    name: String,
    value: Val,
    id: ID,
    relations: Vec<ID>
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "------\nID: {}\nName: {}\nValue: {}\nPoints to: {:?}\n------\n",
                          self.id, self.name, self.value, self.relations)
    }
}

impl Node {
    fn to_bytes(&self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        out.append(&mut Vec::from(self.name.as_bytes()));
        out.append(&mut Vec::from(",".as_bytes()));
        out.append(&mut Vec::from(self.id.to_be_bytes()));
        out.append(&mut Vec::from(",".as_bytes()));
        out.append(&mut self.value.to_bytes());
        out.append(&mut Vec::from(",".as_bytes()));
        for i in self.relations.iter() {
            out.append(&mut Vec::from(i.to_be_bytes()));
            out.append(&mut Vec::from(":".as_bytes()));
        }
        out.append(&mut Vec::from("\n".as_bytes()));
        return out
    }

    fn from_bytes(input: Vec<u8>) -> Self {
        // NodeName,NodeID,NodeValue,relations:relations:\n
        let mut input = input.split(|byte| byte == &(0x2c as u8));
        return Self {
            name: String::from_utf8(input.next().unwrap().to_vec()).unwrap(),
            id: ID::from_be_bytes(input.next().unwrap().try_into().unwrap()),
            value: Val::from_bytes(input.next().unwrap().to_vec()),
            relations: {
                let mut agg: Vec<ID> = Vec::new();
                for i in input.next().unwrap().split(|byte| byte == &(0x3a as u8)) {
                    if !i.is_empty() {
                        agg.push(ID::from_be_bytes(i.try_into().unwrap()));
                    }
                }
                agg
            }
        }
    }
}

#[derive(Clone)]
struct Graph {
    graph: HashMap<ID, Node>,
    counter: ID
}

impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.graph.iter() {
            write!(f, "{}", v)?;
        }
        write!(f, "")
    }
}

impl Graph {
    fn new() -> Self {Graph {graph: HashMap::new(), counter: 0}}
    fn append_node(&mut self, node: Node) {
        if node.id > self.counter {self.counter = node.id}
        self.graph.insert(node.id, node);
    }
    fn append<T: wrappable>(&mut self, name: &str, id: ID, value: T) {
        if id > self.counter {self.counter = id}
        let node = Node {
            name: name.to_string(),
            value: value.wrap(),
            id: id,
            relations: Vec::new()
        };
        self.graph.insert(id, node);
    }
    fn assumed_append<T: wrappable>(&mut self, name: &str, value: T) -> ID {
        self.counter += 1;
        let node = Node {
            name: name.to_string(),
            value: value.wrap(),
            id: self.counter,
            relations: Vec::new()
        };
        self.graph.insert(self.counter, node);
        self.counter
    }
    fn rappend<T: wrappable>(&mut self, name: &str, id: ID, value: T, relations: Vec<ID>) {
        if id > self.counter {self.counter = id}
        let node = Node {
            name: name.to_string(),
            value: value.wrap(),
            id: id,
            relations: relations
        };
        self.graph.insert(id, node);
    }
    fn assumed_rappend<T: wrappable>(&mut self, name: &str, value: T, relations: Vec<ID>) -> ID {
        self.counter += 1;
        let node = Node {
            name: name.to_string(),
            value: value.wrap(),
            id: self.counter,
            relations: relations
        };
        self.graph.insert(self.counter, node);
        self.counter
    }

    fn id_by_name(&self, name: &str) -> Option<ID> {
        for (k, v) in self.graph.iter() {
            if v.name == name.to_string() {return Some(*k)}
        }
        return None
    }

    fn pop_id(&mut self, id: ID) -> Option<Node> {
        self.graph.remove(&id)
    }

    fn change_name(&mut self, id: ID, name: &str) -> Result<(), String> {
        match self.graph.get(&id) {
            Some(_) => {},
            None => return Err("No Such Node!".to_string())
        }
        self.graph.get_mut(&id).unwrap().name = name.to_string();
        Ok(())
    }
    fn change_value<T: wrappable>(&mut self, id: ID, value: T) -> Result<(), String> {
        match self.graph.get(&id) {
            Some(_) => {},
            None => return Err("No Such Node!".to_string())
        }
        self.graph.get_mut(&id).unwrap().value = value.wrap();
        Ok(())
    }
    fn change_relations(&mut self, id: ID, relations: Vec<ID>) -> Result<(), String> {
        match self.graph.get(&id) {
            Some(_) => {},
            None => return Err("No Such Node!".to_string())
        }
        self.graph.get_mut(&id).unwrap().relations = relations;
        Ok(())
    }
    fn add_relation(&mut self, id: ID, relation: ID) -> Result<(), String> {
        match self.graph.get(&id) {
            Some(_) => {},
            None => return Err("No Such Node!".to_string())
        }
        self.graph.get_mut(&id).unwrap().relations.push(relation);
        Ok(())
    }
    fn remove_relation(&mut self, id: ID, relation: ID) -> Result<(), String> {
        match self.graph.get(&id) {
            Some(_) => {},
            None => return Err("No Such Node!".to_string())
        }
        self.graph.get_mut(&id).unwrap().relations.retain(|r| r != &relation);
        Ok(())
    }
}

trait wrappable {
    fn wrap(self) -> Val;
}
impl wrappable for isize  {fn wrap(self) -> Val {Val::Num(self)}}
impl wrappable for String {fn wrap(self) -> Val {Val::Txt(self)}}
impl wrappable for bool   {fn wrap(self) -> Val {Val::Bool(self)}}
impl wrappable for ()     {fn wrap(self) -> Val {Val::None}}

fn main() -> std::io::Result<()> {
    println!("Starting server!");
    let ts = std::time::Instant::now();
    let listener = TcpListener::bind("127.0.0.1:69")?;
    println!("Took {:?}", ts.elapsed());
    println!("Listening on 127.0.0.1:69!");

    let mut socket;
    let mut graph: Graph = Graph::new();
    let mut filename = "".to_string();

    println!("Waiting on connection!");
    loop {
        match listener.accept() {
            Ok((s, _)) => {println!("Connection started"); socket = s; break},
            Err(e) => println!("Error occurred: {e:?}"),
        }
    }
    // Handle Incoming shit
    loop {
        let mut buffer = [0; 1];
        socket.read(&mut buffer)?;
        match buffer[0] {
            0 => {println!("Connection terminated without an exit signal; quitting!"); break},
            1 => { //save as
                let mut length: [u8; 1] = [0; 1];
                socket.read(&mut length)?;
                let mut extra = Vec::new();
                extra.resize(length[0] as usize, 0);
                socket.read(&mut extra[0..length[0] as usize])?;
                filename = String::from_utf8(extra.to_vec()).unwrap();
                let mut writer = BufWriter::new(fs::File::create(&filename)?);
                serialize(&mut writer, graph.clone())?;
                println!("Saved graph in {}", filename);
            },
            2 => { //save
                if filename.is_empty() {
                    println!("No file specified, use save as");
                } else {
                    let mut writer = BufWriter::new(fs::File::create(&filename)?);
                    serialize(&mut writer, graph.clone())?;
                    println!("Saved graph in {}", filename);
                }
            },
            3 => { //open
                let mut length: [u8; 1] = [0; 1];
                socket.read(&mut length)?;
                let mut extra = Vec::new();
                extra.resize(length[0] as usize, 0);
                socket.read(&mut extra[0..length[0] as usize])?;
                filename = String::from_utf8(extra.to_vec()).unwrap();
                graph = deserialize(&filename);
                println!("Read graph from {}", filename);
            },
            4 => { //print
                let printout = format!("{}", graph);
                socket.write(&(printout.len() as u64).to_be_bytes())?;
                socket.write(printout.as_bytes())?;
            },
            255 => { //quit
                println!("Quitting!");
                break
            },
            _ => unreachable!("Yo this should be unreachable, how the fuck")
        }
    }
    Ok(())
}

fn serialize_nodevec<W: std::io::Write>(file: &mut BufWriter<W>, graph: Vec<Node>) -> std::io::Result<()> {
    for i in graph.iter() {
        let bytes = i.to_bytes();
        file.write(&bytes)?;
    }
    file.flush()
}

fn serialize<W: std::io::Write>(file: &mut BufWriter<W>, graph: Graph) -> std::io::Result<()> {
    for (_, i) in graph.graph.iter() {
        let bytes = i.to_bytes();
        file.write(&bytes)?;
    }
    file.flush()
}
fn deserialize_to_nodevec(file: &str) -> Vec<Node> {
    let mut bytes = std::fs::read(file).unwrap();
    let mut out: Vec<Node> = Vec::new();
    for l in bytes.split(|byte| byte == &(0x0a as u8)) {
        if !l.is_empty() {
            out.push(Node::from_bytes(l.to_vec()));
        }
    }
    return out
}

fn deserialize(file: &str) -> Graph {
    let nodevec = deserialize_to_nodevec(file);
    let mut out = Graph::new();
    let mut counter = 0;
    for i in nodevec.iter() {
        if i.id > counter {counter = i.id;}
        out.graph.insert(i.id, i.clone());
    }
    out.counter = counter;
    return out
}

// the misery bin
// why the fuck are the comments in cursive
// this shit is fancier than I could ever be
//
// AAAAAA how I wish rust had function overloading
