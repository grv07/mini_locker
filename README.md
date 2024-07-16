## About

This is an pane/tab locker [Zellij][zellij] plugin in Rust. It can be used to lock indivisual pane or tab.

More about Zellij plugins: [Zellij Documentation][docs]

[zellij]: https://github.com/zellij-org/zellij
[docs]: https://zellij.dev/documentation/plugins.html

## Development

*Note*: you will need to have `wasm32-wasi` added to rust as a target to build the plugin. This can be done with `rustup target add wasm32-wasi`.

## Install
`./install.sh` will create awasm in relaese mode and move the target wasn to `your_config_path/plugins`.

## Otherwise
1. Build the project: `cargo build`
2. Load it inside a running Zellij session: `zellij action start-or-reload-plugin file:target/wasm32-wasi/debug/mini_locker.wasm`
3. Repeat on changes (perhaps with a `watchexec` or similar command to run on fs changes).
