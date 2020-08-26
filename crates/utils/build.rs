use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=colors.csv");
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("colors.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let rdr = BufReader::new(File::open(
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("colors.txt"),
    )?);

    let mut map = phf_codegen::Map::new();
    let mut recs: Vec<(String, String)> = Vec::new();

    for result in rdr.lines() {
        let record = result?;
        let mut record = record.split_whitespace();
        let id = record.next().unwrap().to_owned();
        let hex = record.next().unwrap().to_owned();
        let clean_hex = hex.trim_start_matches('#');
        let color = match clean_hex.len() {
            3 => {
                let d = match u32::from_str_radix(&clean_hex, 16) {
                    Ok(x) => x,
                    Err(_) => 0,
                };

                let r = (d & 0xF) << 4;
                let g = ((d >> 4) & 0xF) << 4;
                let b = ((d >> 8) & 0xF) << 4;

                format!("Color::rgb({}, {}, {})", r, g, b)
            }
            6 => {
                let x = match u32::from_str_radix(&clean_hex, 16) {
                    Ok(x) => x,
                    Err(_) => 0,
                };

                format!(
                    "Color::rgb({}, {}, {})",
                    ((x >> 16) & 0xFF) as u8,
                    ((x >> 8) & 0xFF) as u8,
                    (x & 0xFF) as u8
                )
            }
            _ => panic!(""),
        };
        recs.push((id, color));
    }

    for rec in recs.iter() {
        map.entry(&*rec.0, &rec.1);
    }

    map.entry("transparent", "Color::rgba(0, 0, 0, 0)");

    writeln!(
        &mut file,
        "static COLORS: phf::Map<&'static str, Color> = \n{};\n",
        map.build()
    )
    .unwrap();

    Ok(())
}
