#!/bin/bash

# 设置必要的环境变量
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="${PWD}/target/debug/your_project-%p-%m.profraw"

# 清理旧的覆盖率数据
echo "Cleaning old coverage data..."
cargo +nightly clean

# 运行测试
echo "Running tests..."
cargo +nightly test --target x86_64-unknown-linux-gnu --package co_test
cargo +nightly test --target x86_64-unknown-linux-gnu --package canopen

# 生成覆盖率报告
echo "Generating coverage report..."
grcov ./target/debug/ -s . -t html --llvm --branch --ignore-not-existing --ignore "/*" --binary-path ./target/x86_64-unknown-linux-gnu/debug/deps/ -o ./target/coverage/

# 输出结果
echo "Coverage report generated in ./target/coverage/"
