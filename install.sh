#!/bin/bash

cargo build --release

mv ./target/wasm32-wasi/release/mini_locker.wasm ~/.config/zellij/plugins/
