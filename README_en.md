# chnroutes-rs

[简体中文](./README.md) | English

RIIR version of [chnroutes](https://github.com/fivesheep/chnroutes).

- Locally cached routing information, updated every 7 days
- API calls, fast (windows 1w writes in 30ms)

## Installation

There are several different ways to install chnroutes, you can choose **any of them**.

- Download the file from [Releases](https://github.com/lxl66566/chnroutes-rs/releases), unzip it, and put it in `C:\Windows\System32` (if you are using windows) or any `Path` directory.
- Using [bpm](https://github.com/lxl66566/bpm):
  ```sh
  bpm i chnroutes-rs -b chnroutes -q
  ```
- Using [scoop](https://scoop.sh/):
  ```sh
  scoop bucket add absx https://github.com/absxsfriends/scoop-bucket
  scoop install chnroutes-rs
  ```
- Using cargo:
  ```sh
  cargo +nightly install chnroutes
  ```
  or [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):
  ```sh
  cargo binstall chnroutes
  ```

## Usage

### Command line

```sh
chnroutes export -p windows         # Export routing table manipulation scripts, almost identical to original chnroutes.py (not recommended)
chnroutes up                        # Write routing table items.
chnroutes down                      # Remove routing table items.
```

Since the system API is called directly during `up` and `down` and is very fast, it is recommended to use this method directly instead of the original export script execution.

### Lib

View [examples](./examples)

## Development

```sh
rustup install nightly-2024-04-29
rustup override set nightly-2024-04-29
cargo build --release --bin chnroutes --features=build-binary
```

## TODO

- Source selection (only the original APNIC source is supported at the moment, more sources could be added later)
  - https://github.com/misakaio/chnroutes2
  - https://github.com/Loyalsoldier/geoip
  - https://github.com/oschwald/maxminddb-rust
