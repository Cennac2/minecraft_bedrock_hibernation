# Minecraft Bedrock Hibernation (mbh)

A lightweight proxy that puts your Minecraft Bedrock dedicated server to sleep
when nobody's playing, and wakes it back up automatically the moment a player
tries to connect, so you're not burning resources (or hosting costs) running
a full server 24/7 for a handful of active hours a day.

## How it works

`mbh` sits in front of your real Bedrock server and listens on the port
players actually connect to. As soon as a player tries to join, `mbh`
starts the real Bedrock server process, waits for it to come online, and lets
the connection through.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition or later)
- A Minecraft Bedrock Dedicated Server (BDS) binary, downloaded separately
  from [minecraft.net](https://www.minecraft.net/en-us/download/server/bedrock)

## Installation

### 1. Download a Bedrock Dedicated Server

Download the latest Bedrock Dedicated Server from:

https://www.minecraft.net/en-us/download/server/bedrock

Extract it somewhere convenient, for example:

```text
C:\bedrock_server
```

or

```text
/home/user/bedrock_server
```

### 2. Download or build `mbh`

Either download a release from GitHub or build it yourself using the command below:

```bash
git clone https://github.com/Cennac2/minecraft_bedrock_hibernation.git
cd minecraft_bedrock_hibernation
cargo build --release
```

The executable will be located at:

```text
target/release/minecraft_bedrock_hibernation
```

### 3. Place the executable

You can place `minecraft_bedrock_hibernation` anywhere. It does **not** need to
be inside your Bedrock server directory.

### 4. Run `mbh` once

Start the program:

```bash
./minecraft_bedrock_hibernation
```

On first launch it creates a default `mbh_config.json` in the current working
directory.

### 5. Configure `mbh`

Open `mbh_config.json` and set:

- `bedrock_file_path` to the Bedrock server executable.
- `port` to the port you want players to use for joining.
- `bedrock_server_port` to a port different from `port`.

(Full config file can be found below)

Example:

**Windows**

```json
{
  "bedrock_file_path": "C:/bedrock_server/bedrock_server.exe"
}
```

**Linux**

```json
{
  "bedrock_file_path": "/home/user/bedrock_server/bedrock_server"
}
```

### 6. Start `mbh`

Run:

```bash
./minecraft_bedrock_hibernation
```

Players should now connect to the proxy port (`port`), while the Bedrock server
runs internally on `bedrock_server_port`. The server will automatically start
when someone joins and stop after being idle for the configured amount of time.

## Building

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
  "bedrock_file_path": "./bedrock_server",
  "stop_empty_server_after_seconds": 60
}
```

| Field                  | Description                                                                                          |
|------------------------|------------------------------------------------------------------------------------------------------|
| `port`                            | The public port players connect to (proxy's listening port)                               |
| `bedrock_server_port`             | The port the real Bedrock server runs on internally (should NOT be equal to proxy's port) |
| `protocol_version`                | Bedrock protocol version to advertise; set to `-1` to auto-detect from `version`          |
| `version`                         | Minecraft Bedrock version string shown in the server list and used for protocol version   |
| `hibernating_motd`                | Message shown while the server is asleep                                                  |
| `bedrock_file_path`               | Path to your Bedrock server executable                                                    |
| `stop_empty_server_after_seconds` | How long the server will wait before stopping if there are no players                     |

Edit the file to point `bedrock_file_path` at your actual server binary, then
restart `minecraft_bedrock_hibernation`.
You may also make the `mbh_config.json` from scratch to have the config ready
on first startup.

## Usage

```bash
./minecraft_bedrock_hibernation
```

Players connecting while the server is asleep will trigger it to start
automatically. You can also control it manually from the console:

| Command     | Description                                 |
|-------------|---------------------------------------------|
| `mbh start` | Start the Bedrock server if it's offline    |
| `mbh stop`  | Stop the Bedrock server if it's running     |
| `mbh help`  | List available commands                     |

## Contributing

Contributions are welcome! If you'd like to help out:

1. Fork the repo and add your changes to dev branch
2. Make sure `cargo check` and `cargo build` run clean with no warnings
3. Keep changes focused smaller, single-purpose PRs are easier to review
4. Open a pull request describing what you changed and

## TODO

- [ ] Display a kick message when a player joins while the server is hibernating.

## License

This project is licensed under the [MIT License](LICENSE).
