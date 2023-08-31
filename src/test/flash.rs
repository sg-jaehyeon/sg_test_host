use std::{
    process::{Command, Stdio},
    thread,
};

use crate::{
    app::App,
    jetson::{JetsonModuleType, FlashStatus}
};

const FLASH_SCRIPT: &str = "./tools/kernel_flash/l4t_initrd_flash.sh";

pub fn flash_device(app: &mut App, is_for_test: bool) -> Result<(), ()> {
    let index = app.selected_device_index().unwrap();
    let jetson = &mut app.devlist[index];
    let tx = jetson.create_new_publisher();

    if jetson.is_flashed() || jetson.is_flashing() {
        // TODO send flash skipped signal
        return Ok(());
    }
    
    match jetson.module_type {
        JetsonModuleType::OrinNX16GB => {
            app.flash_status = FlashStatus::Flashing;
            app.flash_handle = Some(thread::spawn(move || {
                let mut child = Command::new(FLASH_SCRIPT)
                                            .current_dir(match is_for_test { true => "./test/Linux_for_Tegra", false => "./release/Linux_for_Tegra"})
                                            .args([
                                                "--flash-only",
                                                "--network",
                                                "usb0",
                                                "--massflash",
                                                "--showlogs",
                                            ])
                                            .stdout(Stdio::piped())
                                            .stderr(Stdio::null())
                                            .spawn()
                                            .unwrap();
                // let stdout = child.stdout.take().unwrap();
                // let reader = BufReader::new(stdout);

                // reader
                //     .lines()
                //     .filter_map(|line| line.ok())
                //     .for_each(|line| { tx.send(line + "\n").unwrap(); });

                child.wait().unwrap();
            }));
        },
        _ => { }
    }

    Ok(())
}