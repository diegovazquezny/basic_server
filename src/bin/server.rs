use std::io::Read;
use std::net::{TcpListener, TcpStream};
fn handle_client(stream: &mut TcpStream) -> std::io::Result<()> {
    let client_address = stream.peer_addr().unwrap();
    println!("{:?} Connected", client_address);
    loop {
        let mut buffer = [0; 1024];
        let bytes = stream.read(&mut buffer[..])?;
        println!("{:?}", bytes);

        if bytes == 0 {
            break;
        }
    }
    println!("{:?} closed", client_address);
    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let _ = handle_client(&mut stream);
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
    Ok(())
}
