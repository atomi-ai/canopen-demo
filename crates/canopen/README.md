# CANOpen for Rust

The library is to provide an embedded use of the canopen rust implementation for devices in need. Therefore, you can understand that this library is `[no_std]`.

## Build and test

### Build
To build the crate in "x86_64-unknown-linux-gnu" and "thumbv6m-none-eabi".
```shell
cargo build --target=x86_64-unknown-linux-gnu
cargo build --target=thumbv6m-none-eabi 
```

### Unit tests
And you can test the project:
```shell
cargo test
```
We still have some unit tests here.

### Integration tests and more
Please refer to [canopen-demo](https://github.com/atomi-ai/canopen-demo) repository to test the libraries.

## Goal and Not-Goal
In the process of implementation, we have some understanding of the canopen protocol, for which we roughly record our goals and not-goals here.


### Goal
CANopen is a nice standard, it's relatively simple and can operate effectively in memory-limited real-time environments. We hope that the library we provide can make it more convenient for more people to provide services in their devices using CANopen.

### Not Goal
#### Thread-safety
This library is not intended for scenarios where a `Node` handles multiple EDS files (devices) simultaneously. If you are using this library in a host computer and want to connect to multiple devices, we recommend you create multiple `Nodes`, each node being used in its own thread. Please remember that our `Node` implementation is not thread-safe. (Because it's unnecessary, as it's not advisable to open several threads on a CANbus to listen and send simultaneously)

#### EDS(DCF) files
Our understanding is that EDS files should be a device's interaction benchmark. The device side (generally appearing as a server) can use the EDS file to initialize its device parameters. However, the problem lies here, EDS files are not a compact file format, and it's very wasteful in terms of flash space and memory in an embedded environment. Therefore, my understanding is that there should be corresponding tools to perform bidirectional conversion of EDS (the same applies to DCF files) to facilitate usage.

#### Persistency
This is a debatable issue that we temporarily place in this section because, as we understand, since we can change parameters and persist through SDO communication, we can also directly change the EDS files to obtain the parameters needed by the device. We reserve discussion on this issue for now, and if we find necessary use cases, we will change this decision.
