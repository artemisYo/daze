# daze
Daze is a graph based database written in rust. It is my first attempt at writing anything like this.
Currently the documentation is not complete enough for me to put it here so.
## Note
I have no fucking idea how databases work. Don't boo me for that, I'm learning.

# Building
Currently daze is split in two, there's the client app, which is the `ui.rs` file right on the highest directory level.
To build it you can simply run `rustc ui.rs`, since it uses no 3rd party dependencies.
It is a client that connects to the server, which you build with `cargo build`, the server open a Tcp listener and as such also requires sudo privileges.
The default ip it opens on is `127.0.0.1:69` and there's currently no way to customize it.
The client also tries to connect to that ip on startup, however asks for a different ip if it fails.
When a connection is established you can use the client to tell the server to execute commands.
Commands like `show` will send a Tcp code to the server, upon which the client expects the server to send back a message;
the message consists of the length of the data transmitted, upon which the data is actually sent.
Commands like `open` work the same way, however are restricted to a string length of 255 bytes, since they use a u8 number, instead of a u64.
The u64 length is sent in the big endian representation.
