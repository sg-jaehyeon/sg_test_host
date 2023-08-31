use std::process::{Stdio, Command};
use std::io::Read;

use crate::App;
use crate::jetson::*;

pub fn refresh_devlist(app: &mut App) {
    use regex::Regex;

    app.clear_devlist();

    let devices = Command::new("bash")
                                .args([
                                    "-c",
                                    "grep 0955 /sys/bus/usb/devices/*/idVendor"
                                ])
                                .stdout(Stdio::piped())
                                .spawn()
                                .unwrap();

    let mut buf = String::new();
    devices.stdout.unwrap().read_to_string(&mut buf).unwrap();

    for device in buf.lines() {
        let re = Regex::new(r"\/sys\/bus\/usb\/devices\/(([\d]+)-([\d.]+))\/idVendor:0955").unwrap();
        let captured = re.captures(device);

        match captured {
            Some(capture) => {
                let path = &capture[1];
                let path = String::from("/sys/bus/usb/devices/") + path + "/";
                let bus = path.clone() + "busnum";
                let dev = path.clone() + "devnum";
                let product = path.clone() + "idProduct";
                
                let busnum = Command::new("cat")
                                .arg(&bus)
                                .output()
                                .unwrap()
                                .stdout;
                let busnum = String::from_utf8(busnum).unwrap().trim().parse::<usize>().unwrap();
                let devnum = Command::new("cat")
                                .arg(&dev)
                                .output()
                                .unwrap()
                                .stdout;
                let devnum = String::from_utf8(devnum).unwrap().trim().parse::<usize>().unwrap();
                let productnum = Command::new("cat")
                                .arg(&product)
                                .output()
                                .unwrap()
                                .stdout;
                let productnum = String::from_utf8(productnum).unwrap().trim().to_string();

                let jetson_detected = Jetson::new(
                    busnum.to_string().as_str(),
                    devnum.to_string().as_str(),
                    "0955",
                    &productnum,
                    &capture[1]
                );

                // 현재 디바이스가 플래시 완료된 상태인지 확인
                let flashed = !["7323", "7423", "7e19", "7523", "7623", "7023", "7223", "7019"].contains(&&jetson_detected.module_number[..]);
                
                if !flashed {
                    app.devlist.push(jetson_detected);
                }
            },
            _ => {
                continue;
            }
        }
    }

    if app.devlist.len() > 0 {
        use crate::UISelectionModel;
        use crate::UISelection;
        app.selection = UISelectionModel { focused: UISelection::DeviceList(None), current: UISelection::DeviceList(Some(0)) };
    }
}
