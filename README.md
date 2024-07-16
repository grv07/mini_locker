## About

This is an pane/tab locker [Zellij][zellij] plugin in Rust. It can be used to lock indivisual pane or tab.

More about Zellij plugins: [Zellij Documentation][docs]

[zellij]: https://github.com/zellij-org/zellij
[docs]: https://zellij.dev/documentation/plugins.html

## Development

*Note*: you will need to have `wasm32-wasi` added to rust as a target to build the plugin. This can be done with `rustup target add wasm32-wasi`.

## Inside Zellij

You can load the `./plugin-dev-workspace.kdl` file as a Zellij layout to get a terminal development environment:

Either when starting Zellij:
```
zellij --layout ./plugin-dev-workspace.kdl
```
*Note that in this case there's a small bug where the plugin is opened twice, it can be remedied by closing the oldest instance or loading with the new-tab action as secified below - this will be addressed in the near future*

Or from a running Zellij session:
```bash
zellij action new-tab --layout ./plugin-dev-workspace.kdl
```

## Install
`./install.sh` will create awasm in relaese mode and move the target wasn to `your_config_path/plugins`.

## Otherwise

1. Build the project: `cargo build`
2. Load it inside a running Zellij session: `zellij action start-or-reload-plugin file:target/wasm32-wasi/debug/mini_locker.wasm`
3. Repeat on changes (perhaps with a `watchexec` or similar command to run on fs changes).
