mod stun;

use std::net::UdpSocket;
use stun::handle_stun_request;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::io::Result;
use signal_hook::{consts::SIGINT, iterator::Signals};

fn main() -> Result<()> {
    // Setup for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Register signal handler for SIGINT
    let mut signals = Signals::new(&[SIGINT]).expect("Error setting signal handler");
    std::thread::spawn(move || {
        for sig in signals.forever() {
            if sig == SIGINT {
                r.store(false, Ordering::SeqCst);
            }
        }
    });

    // Bind the UDP socket to a local address
    let socket = match UdpSocket::bind("0.0.0.0:3478") {
        Ok(s) => {
            println!("Listening on {}", s.local_addr().unwrap());
            s
        },
        Err(e) => {
            eprintln!("Failed to bind to address: {}", e);
            return Err(e);
        }
    };

    socket.set_nonblocking(true).expect("Failed to set socket to non-blocking");

    while running.load(Ordering::SeqCst) {
        let mut buffer = [0; 1024]; // Buffer for incoming data

        // Using `recv_from` with non-blocking error handling
        let (num_bytes, src_addr) = match socket.recv_from(&mut buffer) {
            Ok((num, src)) => (num, src),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => {
                eprintln!("Failed to receive data: {}", e);
                continue;
            }
        };
        println!("Received {} bytes from {}", num_bytes, src_addr);

        // Parse the incoming data as a STUN message
        let request = match stun::StunMessage::decode(&buffer[..num_bytes]) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse STUN message: {}", e);
                continue;
            }
        };

        // Handle the STUN request and generate a response
        let response = match handle_stun_request(&request, &src_addr) {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Failed to handle STUN request: {}", e);
                continue;
            }
        };

        // Send the STUN response back to the client
        let response_bytes = response.encode();
        if let Err(e) = socket.send_to(&response_bytes, src_addr) {
            eprintln!("Failed to send response: {}", e);
        }
    }

    println!("Server shutting down gracefully.");
    Ok(())
}
