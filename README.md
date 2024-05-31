# chnroutes-rs

简体中文 | [English](./README_en.md)

[chnroutes](https://github.com/fivesheep/chnroutes) 的 Rewrite it in Rust 版本。

- 本地缓存路由信息，每 7 天更新
- 调用 API，快速（windows 1w 条写入仅需 30ms）

## 安装

有几种不同方法可以安装此程序，您可以选择其中**任意一种**。

- 在 [Releases](https://github.com/lxl66566/chnroutes-rs/releases) 中下载文件并解压，放入 `C:\Windows\System32`（如果您用的是 windows）或任意 `Path` 目录下。
- 使用 [bpm](https://github.com/lxl66566/bpm)：
  ```sh
  bpm i chnroutes-rs -b chnroutes -q
  ```
- 使用 [scoop](https://scoop.sh/)：
  ```sh
  scoop bucket add absx https://github.com/absxsfriends/scoop-bucket
  scoop install chnroutes-rs
  ```
- 使用 cargo：
  ```sh
  cargo +nightly install chnroutes
  ```
  或 [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)：
  ```sh
  cargo binstall chnroutes
  ```

## 使用

### 命令行

```sh
chnroutes export -p windows         # 导出路由表操作脚本，与原版 chnroutes.py 功能几乎一致（不推荐使用）
chnroutes up                        # 写入路由表项
chnroutes down                      # 移除路由表项
```

由于在 `up` 和 `down` 时直接调用系统 API，速度非常快，建议直接使用此方式，而不是原版的导出脚本执行。

### 库

查看 [examples](./examples)

## 开发

```sh
rustup install nightly-2024-04-29
rustup override set nightly-2024-04-29
cargo build --release --bin chnroutes --features=build-binary
```

## TODO

- 换源（目前仅支持原版 APNIC，日后可添加更多）
  - https://github.com/misakaio/chnroutes2
  - https://github.com/Loyalsoldier/geoip
  - https://github.com/oschwald/maxminddb-rust
- other region support
