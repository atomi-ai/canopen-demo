# Tests for canopen

## Tests for x86 only
We use vcan to test it. Steps as below:

### Create a vcan0 for testing
```shell
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set up vcan0
```

"vcan0" is the fixed name, if it is allocated, you may need to change the code to make the test work.

### Run test
```shell
cd co_test
cargo test
```

## End-to-end test for CANbus with x86 and RP2040
The example client code is in the repo, for test, please refer to canopen-demo repo. 
