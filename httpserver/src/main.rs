mod server;
mod router;
mod handler;

use server::Server;
fn main() {
    let server = Server::new("localhost:8080");
    server.run();
    println!("Hello, world!");
}
