# Where you from?
`whereyoufrom` is a small network diagnostic tool that listens for incoming TCP/UDP connections and simply replies by telling them their IP and port.

# Installation
The recommended way to install is with `cargo` from crates.io:
```
cargo install whereyoufrom
```

Or directly from GitHub:
```
cargo install --git https://github.com/ThomasMiz/whereyoufrom.git whereyoufrom
```

Either one of these will download and compile the tool's code and any dependencies. Once this is done, the executable will become available under the name `whereyoufrom`.

## Downloading binaries
If you don't have `cargo` installed, pre-compiled binaries are available for x84_64 Windows and Linux [in the releases page](https://github.com/ThomasMiz/whereyoufrom/releases).

# Usage
The tool is very simply and straightforward to use:
```
Usage: whereyoufrom [options...]
Options:
  -h, --help                      Display this help menu and exit
  -V, --version                   Display the version number and exit
  -v, --verbose                   Display additional information while running
  -s, --silent                    Do not print to stdout
  -t, --listen-tcp                Specify a TCP socket address to listen for incoming clients
  -u, --listen-udp                Specify a UDP socket address to listen for incoming clients

Socket addresses may be specified as an IPv4 or IPv6 address, or a domainname, and may include a
port number. If no port is specified, then the default of 6969 will be used. If no address is
specified for a transport protocol, then [::] and/or 0.0.0.0 will be used. To disable listening on
a protocol, use "-t -" or "-u -".
```

### Examples
Listens on all IPv4 addresses for UDP with port 6969, but only listens on 192.168.1.105:1234 on TCP:
```
whereyoufrom -t 192.168.1.105:1234
```

Listens only on IPv4 TCP requests coming from this same machine, default port 6969, no UDP:
```
whereyoufrom -t 127.0.0.1 -u -
```
