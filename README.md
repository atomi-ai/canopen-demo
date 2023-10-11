# Demo app for CANopen on RP2040

Given that we need to use CANopen SDO on our RP2040, and we expect to use Rust for our subsequent embedded implementations, we have conducted some research and initiated this project. It includes:

- Can2040(rust): it ports the [can2040](https://github.com/KevinOConnor/can2040) project。
- CANopen(rust): This is an attempt at implementing our own canopen library. Currently, we only enable SDO.
- The demo: Include an experimental server and an example client.

## How to build it
```shell
git clone git@github.com-atomi:zephyr-atomi/canopen_rust.git canopen
cargo build --target thumbv6m-none-eabi
```

## How to run it on a CAN bus
Given our project's goal is to run CANopen on the RP2040, our demo is built directly on the communication between x86 and RP2040. The specific implementation is described below.

### CAN bus structure
x86 <=> Canable USB Adapter <=> SN65HVD230 board <=> RP2040

Canable USB Adapter: [[Amazon](https://www.amazon.com/PRIZOM-Converter-Debugger-Analyzer-Candlelight/dp/B0CD6QFQXH/ref=sr_1_6?crid=2TGJJD1KV2Z36&keywords=CANable&qid=1696911666&sprefix=canable%2Caps%2C353&sr=8-6&th=1)]

SN65HVD230 board: [[Waveshare SN65HVD230](https://www.amazon.com/SN65HVD230-CAN-Board-Communication-Development/dp/B00KM6XMXO/ref=sr_1_2?crid=2I4ZLTIPIB93Q&keywords=SN65HVD230+waveshare&qid=1696911860&sprefix=sn65hvd230+waveshar%2Caps%2C146&sr=8-2)]

For wiring connections, please refer to board manuals / datasheets. We plan to integrate the SN65HVD230 and RP2040 onto a single board in the future for a more comprehensive setup.

### Run the demo

#### Clone dependent repos
```shell
./scripts/clone_repos.sh
```
Please modify the shell script if you've your own repo url. (git submodule need to tediously handle dependent repositories, I decided not to use it in this project)

#### Start the server (target: RP2040)
```shell
cargo run --bin server
```

Logs should look like:
```text
INFO  Program start
└─ server::__cortex_m_rt_main @ src/bin/server.rs:61
INFO  Free bytes in heap: 30860
└─ server::__cortex_m_rt_main @ src/bin/server.rs:104
DEBUG xfguo: can2040_cb 0, msg = CanFrame { id: 234, data: [1, 2, 3, 5, 0, 0, 0, 0] }
└─ can2040::core::can2040_cb @ crates/can2040/src/core.rs:176
```

#### run client (target: x86)
```shell
cargo run --package co_test --target x86_64-unknown-linux-gnu --example client
```
If the program runs correctly, the server window should have received logs corresponding to request/response interactions.
