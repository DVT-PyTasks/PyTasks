use std::io::{self, Write};

use chrono::Local;
use fern::Dispatch;
use log::{debug, error, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

fn setup_logger() -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)   // Set log level
        .chain(std::io::stdout())         // Log to stdout
        .chain(fern::log_file("output.log")?)  // Log to a file
        .apply()?;
    Ok(())
}
#[tokio::main]
async fn main() {
    setup_logger().expect("Failed to initialize logger");
    // Prompt the user to select server or client
    info!("Please select mode: (1) Server, (2) Client");

    // Flush stdout to ensure prompt is displayed before input
    io::stdout().flush().unwrap();

    // Create a mutable string to hold the user input
    let mut input = String::new();

    // Read the user input from stdin
    io::stdin().read_line(&mut input).expect("Failed to read line");

    // Trim the input to remove any newline characters and spaces
    let input = input.trim();

    // Match the input to decide between server and client mode
    match input {
        "1" => {
            info!("Starting server...");
            run_server().await; // Call the function to run the server
        }
        "2" => {
            info!("Starting client...");
            run_client().await.expect("Client error occurred"); // Call the function to run the client
        }
        _ => {
            info!("Invalid selection. Please enter 1 for Server or 2 for Client.");
        }
    }
}

async fn run_server() {
    // Bind to the address
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    info!("Server running on 127.0.0.1:3000");

    loop {
        // Accept incoming connections
        let (mut socket, addr) = listener.accept().await.unwrap();
        info!("New connection from {:?}", addr);

        // Handle the connection
        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];

            // Read data from the socket
            match socket.read(&mut buffer).await {
                Ok(n) => {
                    if n == 0 {
                        return; // Connection closed
                    }

                    println!("Received Request: {}", String::from_utf8_lossy(&buffer[..n]));

                    // Echo the received data back to the client
                    let response = b"Hello, World!";
                    socket.write_all(response).await.unwrap();
                    info!("Sent response to client");
                }
                Err(e) => {
                    error!("Failed to read from socket; err = {:?}", e);
                }
            }
        });
    }
}


async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        info!("Please enter message sent to server");

        // Flush stdout to ensure prompt is displayed before input
        io::stdout().flush().unwrap();

        // Create a mutable string to hold the user input
        let mut input = String::new();

        // Read the user input from stdin
        io::stdin().read_line(&mut input).expect("Failed to read line");

        // Trim the input to remove any newline characters and spaces
        let input = input.trim();
        // Connect to the server
        let mut stream = TcpStream::connect("127.0.0.1:3000").await?;
        println!("Connected to the server");

        // Send a request to the server
        let request = format!("Hello from the client!. Message {}", input);
        stream.write_all(request.as_ref()).await?;
        println!("Sent request to server");

        // Buffer to store the response
        let mut buffer = vec![0; 1024];

        // Read the server's response
        let n = stream.read(&mut buffer).await?;
        println!("Received response: {}", String::from_utf8_lossy(&buffer[..n]));
    }
    Ok(())
}

