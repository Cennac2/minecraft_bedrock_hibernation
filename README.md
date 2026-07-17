# Minecraft Bedrock server Hibernation (mbh)

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![Release](https://img.shields.io/github/v/release/Cennac2/minecraft_bedrock_hibernation?include_prereleases)](https://github.com/Cennac2/minecraft_bedrock_hibernation/releases)


A lightweight proxy that puts your Minecraft Bedrock dedicated server to sleep when nobody's playing, and wakes it back up automatically the moment a player tries to connect so you're not burning resources (or hosting costs) running a full server 24/7 for a handful of active hours a day.
 
If you're paying for a VPS or you host a Bedrock server that only sees a few hours of actual play per day, `mbh` can cut your uptime (and costs) down to just the time people are actually online.

## How it works

`mbh` sits in front of your real Bedrock server and listens on the port
players actually connect to. As soon as a player tries to join, `mbh`
starts the real Bedrock server process, waits for it to come online, and lets
the connection through.

## Requirements

 **Platform**: Windows and Linux
- [Rust](https://www.rust-lang.org/tools/install) (2024 edition or later) (only needed if building from source)
- A Minecraft Bedrock Dedicated Server (BDS) binary, downloaded separately from [minecraft.net](https://www.minecraft.net/en-us/download/server/bedrock)


## Installation

 
### 1. Download a Bedrock Dedicated Server
 
Download the latest Bedrock Dedicated Server from:
 
<https://www.minecraft.net/en-us/download/server/bedrock>
 
Then extract it somewhere convenient

### 2. Get `mbh`
 
**Option A: Download a prebuilt release (recommended)**
 
Grab the latest binary for your platform from the [Releases page](https://github.com/Cennac2/minecraft_bedrock_hibernation/releases).
 
**Option B: Build from source**
 
```
git clone https://github.com/Cennac2/minecraft_bedrock_hibernation.git
cd minecraft_bedrock_hibernation
cargo build --release
```
 
The executable will be located at:
 
```
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

On first launch it creates a default `mbh_config.json` in the current working directory (unless you've already created one yourself, see [Configuration](#configuration)).

### 5. Configure `mbh`

Open `mbh_config.json` and set at minimum:
 
- `bedrock_file_path`: path to the Bedrock server executable
- `port`: the port you want players to use for joining

**Windows**
```json
{
  "bedrock_file_path": "./bedrock_server.exe"
}
```
 
**Linux**
```json
{
  "bedrock_file_path": "./bedrock_server"
}
```


### 6. Start `mbh`

```
./minecraft_bedrock_hibernation
```
 
Players should now connect to the proxy port (`port`), while the Bedrock server runs internally on `bedrock_server_port`. The server will automatically start when someone joins and stop after being idle for the configured amount of time.
 
> **Note:** If your server is reachable over the internet, make sure your router/firewall forwards the **proxy's** port (`port`), not `bedrock_server_port`. Players never connect to the internal port directly.

## Configuration

On first run, `mbh` will generate a default `mbh_config.json` in the working
directory:

```json
{
  "port": 19132,
  "bedrock_server_port": 19134,
  "protocol_version": -1,
  "version": "auto",
  "hibernating_motd": "Server is Hibernating",
  "bedrock_file_path": "./bedrock_server",
  "stop_empty_server_after_seconds": 60
}
```

| Field                              | Description                                                                                |
| ----------------------------------- | -------------------------------------------------------------------------------------------- |
| `port`                              | The public port players connect to (proxy's listening port)                                  |
| `bedrock_server_port`               | The port the real Bedrock server runs on internally (must **not** equal `port`)              |
| `protocol_version`                  | Bedrock protocol version. Set to `-1` to auto-detect; |
| `version`                           | Minecraft Bedrock version string shown to clients. Set to `"auto"` to auto-detect, or hardcode if needed |
| `hibernating_motd`                  | Message shown in the server list while the server is asleep                                  |
| `bedrock_file_path`                 | Path to your Bedrock server executable                                                       |
| `stop_empty_server_after_seconds`   | How long `mbh` waits with no players online before stopping the server                       |
 

Edit the file to point `bedrock_file_path` at your actual server binary, then restart `mbh`. You can also hand-write `mbh_config.json` yourself before the first run so it's ready to go immediately.

## Usage
 
```
./minecraft_bedrock_hibernation
```
 
Players connecting while the server is asleep will trigger it to start automatically. You can also control it manually from the console:


| Command     | Description                                    |
| ----------- | ----------------------------------------------- |
| `mbh start` | Start the Bedrock server if it's offline        |
| `mbh stop`  | Stop the Bedrock server if it's running         |
| `mbh help`  | List available commands                         |
| `mbh exit`  | Fully stop `mbh` (use this instead of Ctrl+C)   |

## Contributing

Contributions are welcome! If you'd like to help out:
 
1. Fork the repo and add your changes to a new branch
2. Make sure `cargo check` and `cargo build` run clean with no warnings
3. Keep changes focused, smaller, single-purpose PRs are easier to review
4. Open a pull request describing what you changed and why
Or if you want to report a bug or request a feature, feel free to do so in the [Issues](https://github.com/Cennac2/minecraft_bedrock_hibernation/issues) tab.


## License

This project is licensed under the [MIT License](LICENSE).
