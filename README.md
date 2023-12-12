# Demo app for CANopen on RP2040

Given that we need to use CANopen SDO on our RP2040, and we expect to use Rust for our subsequent embedded implementations, we have conducted some research and initiated this project.

In this repo, we use:
- [Can2040](https://github.com/atomi-ai/can2040-rust) in Rust: it ports the [can2040](https://github.com/KevinOConnor/can2040) project。
- [CANopen-rust](https://github.com/atomi-ai/canopen-rust): This is an attempt at implementing our own canopen library.

This repo provides an example implementation of the canopen-rust library, as well as the corresponding integration test library. The current example implementation can run on the RP2040. If possible, we will also complete a corresponding demo on the ESP32-C3."

## Build and Test

### Checkout and build
```shell
git clone git@github.com:atomi-ai/canopen-demo.git
cd canopen-demo
cargo build
```

### Integration test
#### Enable "vcan0" for testing
Make sure you've a "vcan0" in your network, or you may need to add it with the command below:
```shell
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set up vcan0
```
You can use "ip link" to check it.

#### Run all test cases
```shell
cargo test --package co_test --target x86_64-unknown-linux-gnu
```

### End-to-End test on "RP2040 <=> x86_64"

#### Wiring structure
x86 <=> Canable USB Adapter <=> SN65HVD230 board <=> RP2040

Canable USB Adapter: [[Amazon](https://www.amazon.com/PRIZOM-Converter-Debugger-Analyzer-Candlelight/dp/B0CD6QFQXH/ref=sr_1_6?crid=2TGJJD1KV2Z36&keywords=CANable&qid=1696911666&sprefix=canable%2Caps%2C353&sr=8-6&th=1)]

SN65HVD230 board: [[Waveshare SN65HVD230](https://www.amazon.com/SN65HVD230-CAN-Board-Communication-Development/dp/B00KM6XMXO/ref=sr_1_2?crid=2I4ZLTIPIB93Q&keywords=SN65HVD230+waveshare&qid=1696911860&sprefix=sn65hvd230+waveshar%2Caps%2C146&sr=8-2)]

For wiring connections, please refer to board manuals / datasheets. We plan to integrate the SN65HVD230 and RP2040 onto a single board in the future for a more comprehensive setup.

TODO(zephyr): Add a wiring diagram here.

#### Enable "can0"
Make sure the USB is up, you can try to use command below to find the Canable is running correctly on "can0". (If it is run on other interface, you may need to modify the code to pass all tests.)
```shell
(You can use tools below)
lsusb  # to check the Canable device is recognized successfully.
sudo dmesg  # To check which device is the Canable device
ip link  # To check the Canable driver works.
```

If "can0" isn't active, please try to activate it. In the server, we're using baud-rate 10000, so please set it the same accordingly.
```shell
sudo ip link set can0 type can bitrate 10000
sudo ip link set up can0
```

If the setting is correct, you can see an active can0 in your network:
```shell
$ ip link
...
22: can0: <NOARP,UP,LOWER_UP,ECHO> mtu 16 qdisc pfifo_fast state UP mode DEFAULT group default qlen 10
    link/can 
```

#### Copy EDS to RP2040 flash.
We wrote some code to copy EDS file through RP2040 uart port. On the RP2040 side, we run command below to wait for the file and write it to flash.
```shell
cargo run --target thumbv6m-none-eabi --bin read_uart_and_write_flash
```
And on the x86_64 machine, we run the script below to upload the EDS file.
```shell
python scripts/send_file_over_uart.py /dev/ttyACM1 ./co_test/tests/fixtures/demoDevice.eds
```
Remember to wire the RP2040 uart pin to a usb-uart device.

// TODO(zephyr): My wiring details.

#### Start the node on RP2040
```shell
cargo run --target thumbv6m-none-eabi --bin server
```

#### Run the test client on x86 machine
```shell
cargo run --package co_test --target x86_64-unknown-linux-gnu --bin client
```
If the program runs correctly, the server window should have received logs corresponding to request/response interactions.

### Coverage
```shell
./scripts/run_coverage.sh
```
And you can check the coverage result in target/coverage.


## References
// TODO(zephyr): Add docs into the repo.
- [CiA 301](docs/301_canopen.pdf): The CANOpen protocol standard.
- [CiA 306](docs/dsp306.pdf): EDS/DCF file standard.
- [A canopen in one page](docs/canopen_poster.pdf)

## TODO
- [ ] Enable canopen on ESP32.
