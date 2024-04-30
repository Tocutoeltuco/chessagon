use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = env::var("OUT_DIR")?;
    let out = Path::new(&out);
    let src = Path::new("data");

    gen(src.join("name-left.txt"), out.join("name-left.rs"))?;
    gen(src.join("name-right.txt"), out.join("name-right.rs"))?;

    Ok(())
}

fn gen(src_path: impl AsRef<Path>, out_path: impl AsRef<Path>) -> io::Result<()> {
    let src = BufReader::new(File::open(src_path.as_ref())?);
    let mut out = BufWriter::new(File::create(out_path.as_ref())?);

    writeln!(out, "[")?;
    for word in src.lines() {
        writeln!(out, "\"{}\",", &word.unwrap())?;
    }
    writeln!(out, "]")
}
