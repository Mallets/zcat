# zat - the zenoh cat

`zat` mimics [netcat](https://sectools.org/tool/netcat/) with the difference that communication occurs via [zenoh](https://github.com/eclipse-zenoh/zenoh) instead of plain TCP or UDP.

`zat` allows to read and write data across networks from the command line.

## Prerequisites

`zat` is written in Rust and requires the [Rust toolchain](https://www.rust-lang.org/tools/install) to be installed on your system to build it.

## Installation

Install the `zat` command using `cargo`. 

```sh
cargo install zat
```

## Usage

See `zat --help` for all available options.

### Getting started

`zat` runs in publicatoin or subscription mode when proper flags are
specified.

To read data from stdin and publish it over zenoh:

```sh
echo "Hello World" | zat -w foo/bar
```

To subscribe to data from zenoh and write it to stdout:

```sh
zat -r foo/bar
```

### QoS parameters

The QoS parameters of zenoh publications can be configured via command line per every key expression:

```
<keyexpr>:<drop|block>?:<priority>?:<true|false>?
```

To change the reliability:
```sh
echo "Hello World" | zat -w foo/bar:besteffort
```

To change the reliability and the congestion control:
```sh
echo "Hello World" | zat -w foo/bar:besteffort:drop
```

To change the reliability, the congestion control and the priority:
```sh
echo "Hello World" | zat -w foo/bar:besteffort:drop:6
```

To change the reliability, the congestion control, the priority, and the express flag:
```sh
echo "Hello World" | zat -w foo/bar:besteffort:drop:6:true
```

### Custom Zenoh Configuration

A zenoh configuration file can be provided in the command line.

```sh
zat --config config.json5 -w foo/bar
```

To listen for incoming connections:

```sh
zat -l tcp/127.0.0.1:6777 -w foo/bar
```

To establish a connection:

```sh
zat -e tcp/127.0.0.1:7447 -w foo/bar
```

More Zenoh options can be discovered by `zat --help`.

## License

The software is distributed with a Eclipse Public License v2.0 license. 
You can read the [license file](LICENSE.txt).