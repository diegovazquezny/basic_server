use std::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    loop {
        let mut buffer = String::new();
        println!("What is your name? ");
        io::stdin().read_line(&mut buffer)?;
        stream.write_all_buf(&mut buffer.as_bytes()).await?;
        if &buffer == "quit\n" {
            break;
        }
    }
    Ok(())
}
// TODO: get user name and save it somewhere on the server associating it with the socket address
// TODO: allow clients to receive messages from other users, showing the name instead of the socket
// address
async fn set_name(stream: &mut TcpStream) -> tokio::io::Result<()> {
    let mut buffer = String::new();
    println!("What is your name? ");
    io::stdin().read_line(&mut buffer)?;
    stream.write_all_buf(&mut buffer.as_bytes()).await?;
    Ok(())
}
