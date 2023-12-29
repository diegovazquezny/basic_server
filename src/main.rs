use ::std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::SystemTime;

#[derive(Debug)]
struct Message {
    message: String,
    send_at: SystemTime,
}

type MessageVector = Vec<Message>;

type ClientsMap = HashMap<SocketAddr, MessageVector>;

fn save_message(
    client_address: &SocketAddr,
    message: &String,
    clients_map: &mut ClientsMap,
) -> std::io::Result<()> {
    println!("{}, {}, {:?}", client_address, message, clients_map);
    // if first message, insert (client_address, empty MessageVector)
    if !clients_map.contains_key(client_address) {
        clients_map.insert(*client_address, Vec::new());
    }
    // get MessageVector using key and insert message
    //let mut vector = clients_map.get(client_address).unwrap();
    clients_map.entry(*client_address).and_modify(|vec| {
        vec.push(Message {
            message: message.to_string(),
            send_at: SystemTime::now(),
        })
    });
    println!("{:?}", clients_map);
    Ok(())
}

fn handle_client(stream: &mut TcpStream) -> std::io::Result<()> {
    let client_address = stream.peer_addr().unwrap();
    //let mut clients_map: ClientsMap = HashMap::new();
    println!("{:?} Connected", client_address);
    loop {
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer[..])?;
        let filtered_buffer: Vec<u8> = buffer
            .iter()
            .filter_map(|x| if *x > 0 { Some(*x) } else { None })
            .collect();
        println!("filtered_buffer {:?}", filtered_buffer);
        let string = String::from_utf8(filtered_buffer).unwrap();
        //save_message(&client_address, &string, &mut clients_map).expect("Failed to save message");
        println!("After reading from stream {:?}", string);
        // only send message to other clients
        stream.write_all(string.as_bytes())?;

        if string.is_empty() {
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
        println!("Hello listener: {:?}", listener);
        match stream {
            Ok(mut stream) => {
                let _ = handle_client(&mut stream);
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
    Ok(())
}
// TODO:
// send message to all chat clients except the sender
// store the client addresses in a vector
// figure out why server crashes when client ^c to exit
// save client messages and show if client types in "--history"
// handle errors gracefully instead of using unwrap
