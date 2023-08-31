use std::io;
use std::process::Command;

pub fn compile_to_file(dts: &str, dtb: &str) -> io::Result<()> {
    let _ = Command::new("dtc")
                        .args([
                            "-I",
                            "dts",
                            "-O",
                            "dtb",
                            dts,
                            "-o",
                            dtb
                        ])
                        .output()?;
    Ok(())
}