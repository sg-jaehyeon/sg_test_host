use std::io;
use std::process::Command;
use std::sync::mpsc::Sender;
use crate::jetson::Signal;

pub fn decompile_to_file(dtb: &str, dts: &str) -> io::Result<()> {
    let _ = Command::new("dtc")
                        .args([
                            "-I",
                            "dtb",
                            "-O",
                            "dts",
                            dtb,
                            "-o",
                            dts,
                        ])
                        .output()?;

    Ok(())
}

pub fn decompile_to_string(_tx: Sender<Signal>, dtb: &str) -> io::Result<String> {    
    let output = Command::new("dtc")
                    .args([
                        "-q",
                        "-I",
                        "dtb",
                        "-O",
                        "dts",
                        dtb,
                    ])
                    .output()
                    .unwrap();
    
    let output = String::from_utf8(output.stdout).unwrap();

    Ok(output)
}