use std::net::SocketAddr;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

#[derive(Debug, Clone)]
struct Message {
    message: String,
    socket_address: SocketAddr,
}
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let (sender, _receiver) = broadcast::channel(16);

    loop {
        let (mut stream, socket_address) = listener.accept().await.unwrap();
        let sender = sender.clone();
        let mut receiver = sender.subscribe();

        tokio::spawn(async move {
            println!("Accepted connection from: {:?}", socket_address);
            if let Err(e) = process(&mut stream, &mut receiver, &sender, &socket_address).await {
                println!("An error occurred while accepting a connection: {:?}", e);
            }
        });
    }
}

async fn process(
    stream: &mut TcpStream,
    receiver: &mut Receiver<Message>,
    sender: &Sender<Message>,
    socket_address: &SocketAddr,
) -> io::Result<()> {
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
                // send attempts to send a value to all active receivers
                let _subscribed_receivers = sender.send(message).unwrap();
            }
            // receive the broadcasted messages and write to the stream
            result = receiver.recv() => {
                let message_struct = result.unwrap();
                let Message {message, socket_address: message_socket_address} = message_struct;
                let formatted_message = format!("{}:: {}", message_struct.socket_address, message);
                if message_socket_address != *socket_address {
                    stream_writer.write_all(formatted_message.to_string().as_bytes()).await?;
                } else {
                    print!("{}", formatted_message);
                }
            }
        }
    }
    Ok(())
}
