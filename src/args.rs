use std::{
    env, fmt,
    io::ErrorKind,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs},
};

pub const DEFAULT_PORT: u16 = 6969;

pub fn get_version_string() -> String {
    format!(
        concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"), " ({} {})"),
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

pub fn get_help_string() -> &'static str {
    concat!(
        "Usage: whereyoufrom [options...]\n",
        "Options:\n",
        "  -h, --help                      Display this help menu and exit\n",
        "  -V, --version                   Display the version number and exit\n",
        "  -v, --verbose                   Display additional information while running\n",
        "  -s, --silent                    Do not print to stdout\n",
        "  -t, --listen-tcp                Specify a TCP socket address to listen for incoming clients\n",
        "  -u, --listen-udp                Specify a UDP socket address to listen for incoming clients\n",
        "\n",
        "Socket addresses may be specified as an IPv4 or IPv6 address, or a domainname, and may include a port number. If ",
        "no port is specified, then the default of 6969 will be used. If no address is specified for a transport protocol, ",
        "then [::] and/or 0.0.0.0 will be used. To disable listening on a protocol, use \"-t -\" or \"-u -\".\n",
        "\n",
        "\n",
        "Examples:\n",
        "Listens on all IPv4 addresses for UDP with port 6969, but only listens on 192.168.1.105:1234 on TCP:\n",
        "    whereyoufrom -t 192.168.1.105:1234\n",
        "\n",
        "Listens only on IPv4 TCP requests coming from this same machine, default port 6969, no UDP:\n",
        "    whereyoufrom -t 127.0.0.1 -u -\n",
        "\n",
        "Author: Thomas Mizrahi\n",
    )
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArgumentsRequest {
    Help,
    Version,
    Run(StartupArguments),
}

#[derive(Debug, PartialEq, Eq)]
pub struct StartupArguments {
    pub verbose: bool,
    pub silent: bool,
    pub tcp_addresses: Vec<SocketAddr>,
    pub udp_addresses: Vec<SocketAddr>,
}

impl StartupArguments {
    pub fn empty() -> Self {
        StartupArguments {
            verbose: false,
            silent: false,
            tcp_addresses: Vec::new(),
            udp_addresses: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArgumentsError {
    UnknownArgument(String),
    TcpListenError(SocketErrorType),
    UdpListenError(SocketErrorType),
    NoSocketsSpecified,
}

impl fmt::Display for ArgumentsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownArgument(arg) => write!(f, "Unknown argument: {arg}"),
            Self::TcpListenError(tcp_error) => tcp_error.fmt(f),
            Self::UdpListenError(udp_error) => udp_error.fmt(f),
            Self::NoSocketsSpecified => write!(f, "No sockets were specified for TCP nor UDP!"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SocketErrorType {
    UnexpectedEnd(String),
    InvalidSocketAddress(String, String),
}

impl fmt::Display for SocketErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd(arg) => write!(f, "Expected socket address after {arg}"),
            Self::InvalidSocketAddress(arg, addr) => write!(f, "Invalid socket address after {arg}: {addr}"),
        }
    }
}

fn parse_socket_arg(
    result_vec: &mut Vec<SocketAddr>,
    arg: String,
    maybe_arg2: Option<String>,
    default_port: u16,
) -> Result<(), SocketErrorType> {
    let arg2 = match maybe_arg2 {
        Some(value) => value,
        None => return Err(SocketErrorType::UnexpectedEnd(arg)),
    };

    let iter = match arg2.to_socket_addrs() {
        Ok(iter) => iter,
        Err(err) if err.kind() == ErrorKind::InvalidInput => match format!("{arg2}:{default_port}").to_socket_addrs() {
            Ok(iter) => iter,
            Err(_) => return Err(SocketErrorType::InvalidSocketAddress(arg, arg2)),
        },
        Err(_) => return Err(SocketErrorType::InvalidSocketAddress(arg, arg2)),
    };

    for sockaddr in iter {
        if !result_vec.contains(&sockaddr) {
            result_vec.push(sockaddr);
        }
    }

    Ok(())
}

pub fn parse_arguments<T>(mut args: T) -> Result<ArgumentsRequest, ArgumentsError>
where
    T: Iterator<Item = String>,
{
    let mut result = StartupArguments::empty();

    // Ignore the first argument, as it's by convention the name of the program
    args.next();

    let mut tcp_specified = false;
    let mut udp_specified = false;

    while let Some(arg) = args.next() {
        if arg.is_empty() {
            continue;
        } else if arg.eq("-h") || arg.eq_ignore_ascii_case("--help") {
            return Ok(ArgumentsRequest::Help);
        } else if arg.eq("-V") || arg.eq_ignore_ascii_case("--version") {
            return Ok(ArgumentsRequest::Version);
        } else if arg.eq("-v") || arg.eq_ignore_ascii_case("--verbose") {
            result.verbose = true;
        } else if arg.eq("-s") || arg.eq_ignore_ascii_case("--silent") {
            result.silent = true;
        } else if arg.eq("-t") || arg.eq_ignore_ascii_case("--listen-tcp") {
            tcp_specified = true;
            let arg2 = args.next();
            if !arg2.as_deref().is_some_and(|s| s.trim() == "-") {
                parse_socket_arg(&mut result.tcp_addresses, arg, arg2, DEFAULT_PORT).map_err(ArgumentsError::TcpListenError)?;
            }
        } else if arg.eq("-u") || arg.eq_ignore_ascii_case("--listen-udp") {
            udp_specified = true;
            let arg2 = args.next();
            if !arg2.as_deref().is_some_and(|s| s.trim() == "-") {
                parse_socket_arg(&mut result.udp_addresses, arg, arg2, DEFAULT_PORT).map_err(ArgumentsError::UdpListenError)?;
            }
        } else {
            return Err(ArgumentsError::UnknownArgument(arg));
        }
    }

    if !tcp_specified {
        result
            .tcp_addresses
            .push(SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, DEFAULT_PORT, 0, 0)));
        result
            .tcp_addresses
            .push(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DEFAULT_PORT)));
    }

    if !udp_specified {
        result
            .udp_addresses
            .push(SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, DEFAULT_PORT, 0, 0)));
        result
            .udp_addresses
            .push(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DEFAULT_PORT)));
    }

    if result.udp_addresses.is_empty() && result.tcp_addresses.is_empty() {
        return Err(ArgumentsError::NoSocketsSpecified);
    }

    Ok(ArgumentsRequest::Run(result))
}
