use std::{env, fs::File, io::Write, path::Path};

use zstd::stream::write::Encoder;

fn main() {
    println!("cargo:rerun-if-changed=build.rs,assets");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("apnic.zst");

    let mut encoder = Encoder::new(Vec::new(), 22).unwrap();
    encoder.write_all(include_bytes!("assets/apnic")).unwrap();
    let compressed_bytes = encoder.finish().unwrap();

    let mut f = File::create(dest_path).unwrap();
    f.write_all(&compressed_bytes).unwrap();
}
