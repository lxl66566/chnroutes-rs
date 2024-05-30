# chnroutes-rs

WIP.

简体中文 | [English](./README_en.md)

[chnroutes](https://github.com/fivesheep/chnroutes) 的 rust 版本。

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
chnroutes export -p windows         # 与原版 chnroutes 功能相同
chnroutes up                        # 写入路由表
chnroutes down                      # 移除路由表
```

### 库

```rs
use chnroutes::{Result, Source, Target};
fn main() -> Result<()> {
    let cn_ip_results: Vec<ipnet::Ipv4Net> = chnroutes::source::apnic::fetch_ip_data()?;
    let user_script: Result<(String, Option<String>)> = Target::Linux.export_str(&Source::apnic);
    Ok(())
}
```

## 特性

- 缓存路由信息，每 7 天更新
- 可换源（目前仅支持原版 APNIC，日后可添加更多）
