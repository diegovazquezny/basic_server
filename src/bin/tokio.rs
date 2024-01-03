use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Clone)]
struct Message {
    message: String,
    socket_address: SocketAddr,
}

#[derive(Debug, Clone)]
struct UserData {
    messages: Vec<Message>,
    user_name: String,
}

#[tokio::main]
async fn main() {
    let hash_map: HashMap<SocketAddr, UserData> = HashMap::new();
    let user_map = Arc::new(Mutex::new(hash_map));
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let (tx, _rx) = broadcast::channel(16);

    loop {
        let (mut stream, socket_address) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let user_map = Arc::clone(&user_map);
        tokio::spawn(async move {
            println!("Accepted connection from: {:?}", socket_address);
            if let Err(e) = get_user_name(&mut stream, &socket_address, &user_map).await {
                eprintln!(
                    "An error occurred while getting user name from {}. {}",
                    socket_address, e
                );
            }
            if let Err(e) = process(&mut stream, &mut rx, &tx, &socket_address, user_map).await {
                eprintln!("An error occurred while accepting a connection: {:?}", e);
            }
        });
    }
}

async fn process(
    stream: &mut TcpStream,
    rx: &mut Receiver<Message>,
    tx: &Sender<Message>,
    socket_address: &SocketAddr,
    user_map: Arc<Mutex<HashMap<SocketAddr, UserData>>>,
) -> io::Result<()> {
    let user_map = Arc::clone(&user_map);
    let (stream_reader, mut stream_writer) = stream.split();
    let mut stream_reader = BufReader::new(stream_reader);

    loop {
        let mut line = String::new();
        tokio::select! {
            result = stream_reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                 break;
                }
                let message = Message {
                    message: line,
                    socket_address: *socket_address,
                };
                let mut unlocked_user_map = user_map.lock().await;
                let user_data = unlocked_user_map.get_mut(socket_address).unwrap();
                let messages = &mut user_data.messages;
                messages.push(message.clone());
                println!("{:?}", unlocked_user_map.get(socket_address));
                // get a mut reference to the messages vector inside the struct
                // send attempts to send:write!(, "") a value to all active receivers
                let _subscribed_receivers = tx.send(message).unwrap();
            }
            // receive the broadcasted messages and write to the stream
            result = rx.recv() => {
                let message_struct = result.unwrap();
                let Message {message, socket_address: message_socket_address} = message_struct;
                let unlocked_user_map = user_map.lock().await;
                let user_data = unlocked_user_map.get(&message_socket_address).unwrap();
                let user_name = user_data.user_name.to_string();
                let formatted_message = format!("{}:: {}", &user_name.trim_end_matches("\r\n"), message);
                if message_socket_address != *socket_address {
                    stream_writer.write_all(formatted_message.to_string().as_bytes()).await?;
                    print!("{}", formatted_message);
                }
            }
        }
    }
    Ok(())
}

async fn get_user_name(
    stream: &mut TcpStream,
    socket_address: &SocketAddr,
    user_map: &Arc<Mutex<HashMap<SocketAddr, UserData>>>,
) -> io::Result<()> {
    let (stream_reader, mut stream_writer) = stream.split();
    let mut stream_reader = BufReader::new(stream_reader);
    let mut line = String::new();
    stream_writer
        .write_all(b"What is your user name?\n")
        .await?;
    stream_reader.read_line(&mut line).await?;
    let welcome_message = format!("Welcome to the chat {}\n", line);
    stream_writer.write_all(welcome_message.as_bytes()).await?;
    let mut unlocked_user_map = user_map.lock().await;
    unlocked_user_map.insert(
        *socket_address,
        UserData {
            messages: vec![],
            user_name: line,
        },
    );
    Ok(())
}
// TODO: save messages in hashmap
// TODO: work on client
