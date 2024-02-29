use std::{
    io::{Cursor, Write},
    net::SocketAddr,
    process::exit,
};

use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, UdpSocket},
};

use crate::{args::StartupArguments, printlnif};

pub const UDP_BUF_SIZE: usize = 1400;

pub async fn run_server(startup_args: StartupArguments) {
    let tcp_listeners = bind_tcp_listeners(startup_args.verbose, &startup_args.tcp_addresses);
    let udp_sockets = bind_udp_sockets(startup_args.verbose, &startup_args.udp_addresses);

    if tcp_listeners.is_empty() && udp_sockets.is_empty() {
        eprintln!("ERROR! No TCP nor UDP sockets could be bound. Aborting.");
        exit(1);
    }

    if !startup_args.tcp_addresses.is_empty() && tcp_listeners.is_empty() {
        eprintln!("WARNING! No TCP sockets were bound!");
    }

    if !startup_args.udp_addresses.is_empty() && udp_sockets.is_empty() {
        eprintln!("WARNING! No UDP sockets were bound!");
    }

    let mut handles = Vec::with_capacity(tcp_listeners.len() + udp_sockets.len());

    handles.extend(tcp_listeners.into_iter().map(|listener| {
        tokio::task::spawn_local(async move {
            run_tcp_server(startup_args.verbose, startup_args.silent, listener).await;
        })
    }));

    handles.extend(udp_sockets.into_iter().map(|socket| {
        tokio::task::spawn_local(async move {
            run_udp_server(startup_args.verbose, startup_args.silent, socket).await;
        })
    }));

    let _ = tokio::signal::ctrl_c().await;
    printlnif!(!startup_args.silent, "Received break signal, shutting down");
    for handle in handles {
        handle.abort();
    }
}

fn bind_tcp_listeners(verbose: bool, addresses: &Vec<SocketAddr>) -> Vec<TcpListener> {
    let mut tcp_listeners = Vec::new();
    for addr in addresses {
        printlnif!(verbose, "Binding TCP socket at {addr}");

        let std_listener = match std::net::TcpListener::bind(addr) {
            Ok(l) => l,
            Err(error) => {
                eprintln!("Failed to bind TCP socket at {addr}: {error}");
                continue;
            }
        };

        if let Err(error) = std_listener.set_nonblocking(true) {
            eprintln!("Failed to set TCP socket {addr} as nonblocking: {error}");
            continue;
        }

        let listener = match TcpListener::from_std(std_listener) {
            Ok(l) => l,
            Err(error) => {
                eprintln!("Failed to convert `std::net::TcpListener` into `tokio::net::TcpListener`: {error}");
                continue;
            }
        };

        printlnif!(verbose, "Successfully bound TCP socket at {addr}");
        tcp_listeners.push(listener)
    }

    tcp_listeners
}

fn bind_udp_sockets(verbose: bool, addresses: &Vec<SocketAddr>) -> Vec<UdpSocket> {
    let mut udp_sockets = Vec::new();
    for addr in addresses {
        printlnif!(verbose, "Binding UDP socket at {addr}");

        let std_socket = match std::net::UdpSocket::bind(addr) {
            Ok(s) => s,
            Err(error) => {
                eprintln!("Failed to bind UDP socket at {addr}: {error}");
                continue;
            }
        };

        if let Err(error) = std_socket.set_nonblocking(true) {
            eprintln!("Failed to set UDP socket {addr} as nonblocking: {error}");
            continue;
        }

        let socket = match UdpSocket::from_std(std_socket) {
            Ok(s) => s,
            Err(error) => {
                eprintln!("Failed to convert `std::net::UdpSocket` into `tokio::net::UdpSocket`: {error}");
                continue;
            }
        };

        printlnif!(verbose, "Successfully bound UDP socket at {addr}");
        udp_sockets.push(socket)
    }

    udp_sockets
}

async fn run_tcp_server(verbose: bool, silent: bool, listener: TcpListener) {
    let addr = listener.local_addr().unwrap();

    let mut counter = 0u64;
    let mut error_counter = 0;

    loop {
        counter += 1;
        let (mut stream, remote_address) = match listener.accept().await {
            Ok(t) => t,
            Err(error) => {
                printlnif!(!silent, "Error while accepting from TCP socket {addr}: {error}");
                error_counter += 1;
                if error_counter >= 10 {
                    break;
                }
                continue;
            }
        };
        printlnif!(!silent, "TCP listener {addr} accepted connection from {remote_address}");

        tokio::task::spawn_local(async move {
            let mut buf = [0u8; 256];
            let mut cursor = Cursor::new(buf.as_mut());
            let _ = write!(cursor, "you: {remote_address} | connection_number: {counter}");

            match stream.write_all(&buf).await {
                Ok(()) => {
                    printlnif!(
                        verbose,
                        "TCP socket {addr} responded to {remote_address} with connection number {counter}"
                    )
                }
                Err(error) => {
                    eprintln!("TCP socket {addr} failed to respond to {remote_address}: {error}");
                }
            }

            let _ = stream.shutdown().await;
        });
    }
    eprintln!("TCP socket {addr} closed due to too many consecutive errors.");
}

async fn run_udp_server(verbose: bool, silent: bool, socket: UdpSocket) {
    let addr = socket.local_addr().unwrap();
    let mut buf = [0u8; UDP_BUF_SIZE];

    let mut counter = 0u64;
    let mut error_counter = 0;

    loop {
        counter += 1;
        let (buf_len, remote_address) = match socket.recv_from(&mut buf).await {
            Ok(t) => {
                error_counter = 0;
                t
            }
            Err(error) => {
                printlnif!(!silent, "Error while receiving from UDP socket {addr}: {error}");
                error_counter += 1;
                if error_counter >= 10 {
                    break;
                }
                continue;
            }
        };

        printlnif!(!silent, "UDP socket {addr} received {buf_len} bytes from {remote_address}");
        let mut cursor = Cursor::new(buf.as_mut());
        let _ = write!(cursor, "you: {remote_address} | bytes: {buf_len} | packet_number: {counter}");
        let len = cursor.position() as usize;

        match socket.send_to(&buf[..len], remote_address).await {
            Ok(bytes_sent) if bytes_sent != len => {
                eprintln!("UDP socket {addr} should have sent {len} bytes to {remote_address}, but {bytes_sent} were sent")
            }
            Ok(_) => printlnif!(
                verbose,
                "UDP socket {addr} responded to {remote_address} with packet number {counter}"
            ),
            Err(error) => eprintln!("UDP socket {addr} failed to respond to {remote_address}: {error}"),
        };
    }

    eprintln!("UDP socket {addr} closed due to too many consecutive errors.");
}
