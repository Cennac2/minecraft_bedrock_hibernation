# Minecraft Bedrock Hibernation (mbh)

A lightweight proxy that puts your Minecraft Bedrock dedicated server to sleep
when nobody's playing, and wakes it back up automatically the moment a player
tries to connect — so you're not burning resources (or hosting costs) running
a full server 24/7 for a handful of active hours a day.

## How it works

`mbh` sits in front of your real Bedrock server and listens on the port
players actually connect to. As soon as a player tries to join, `mbh`
starts the real Bedrock server process, waits for it to come online, and lets
the connection through.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition or later)
- A Minecraft Bedrock Dedicated Server (BDS) binary, downloaded separately
  from [minecraft.net](https://www.minecraft.net/en-us/download/server/bedrock)

## Installation

```bash
git clone https://github.com/Cennac2/minecraft_bedrock_hibernation.git
cd minecraft_bedrock_hibernation
cargo build --release
```

The compiled binary will be at `target/release/minecraft_bedrock_hibernation`.

## Configuration

On first run, `mbh` will generate a default `mbh_config.json` in the working
directory:

```json
{
  "port": 19132,
  "bedrock_server_port": 19134,
  "protocol_version": -1,
  "version": "1.26.30",
  "hibernating_motd": "Server is Hibernating",
  "bedrock_file_path": "./bedrock_server"
}
```

| Field                  | Description                                                                               |
|------------------------|-------------------------------------------------------------------------------------------|
| `port`                 | The public port players connect to (proxy's listening port)                               |
| `bedrock_server_port`  | The port the real Bedrock server runs on internally (should NOT be equal to proxy's port) |
| `protocol_version`     | Bedrock protocol version to advertise; set to `-1` to auto-detect from `version`          |
| `version`              | Minecraft Bedrock version string shown in the server list                                 |
| `hibernating_motd`     | Message shown while the server is asleep                                                  |
| `bedrock_file_path`    | Path to your Bedrock server executable                                                    |

Edit the file to point `bedrock_file_path` at your actual server binary, then
restart `minecraft_bedrock_hibernation`.

## Usage

```bash
./minecraft_bedrock_hibernation
```

Players connecting while the server is asleep will trigger it to start
automatically. You can also control it manually from the console:

| Command     | Description                                |
|-------------|---------------------------------------------|
| `mbh start` | Start the Bedrock server if it's offline    |
| `mbh stop`  | Stop the Bedrock server if it's running     |
| `mbh help`  | List available commands                     |

## Contributing

Contributions are welcome! If you'd like to help out:

1. Fork the repo and create a branch for your change
2. Make sure `cargo check` and `cargo build` run clean with no warnings
3. Keep changes focused smaller, single-purpose PRs are easier to review
4. Open a pull request describing what you changed and why

If you're planning a bigger change, feel free to open an issue first to
discuss the approach before putting in the work.

## TODO

- [ ] Harden locking around auto-start so a player connecting and `mbh start`
      can't race each other into starting the server twice.
- [ ] Configurable idle timeout to auto-stop the server after a period of
      no players.
- [ ] Tests around config parsing and the hibernation/wake flow.

## License

This project is licensed under the [MIT License](LICENSE).
