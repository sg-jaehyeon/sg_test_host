use std::{sync::mpsc::Sender, time::SystemTime};

use crate::logger::Logger;

pub enum JetsonModuleType {
    OrinNX16GB,
    OrinNX8GB,
    OrinNano8GB,
    OrinNano4GB,
    XavierNX,
    Unknown,
    None,
}

#[derive(PartialEq)]
pub enum Signal {
    Message(String),
    FlashFail,
    FlashSuccess,
    FlashPass,
    Flashing,
    EnvironmentInstalling(SystemTime),
    EnvironmentPass,
    EnvironmentInstalled,
}

#[derive(PartialEq)]
pub enum FlashStatus {
    Wait,
    Flashing,
    Finished,
    Failed,
}

pub struct Jetson {
    pub bus: String,
    pub dev: String,
    pub vendor_number: String,
    pub module_number: String,
    pub module_name: String,
    pub module_type: JetsonModuleType,
    pub instance_number: String,
    pub ip_v4: Option<String>,
    pub logger: Option<Logger>,
    pub status: FlashStatus,
}

pub const XAVIER_NX_DTB: &'static str = "tegra194-p3668-0001-p3509-0000.dtb";
pub const XAVIER_NX_DTS: &'static str = "tegra194-p3668-0001-p3509-0000.dts";
pub const ORIN_NX_16GB_DTB: &'static str = "tegra234-p3767-0000-p3768-0000-a0.dtb";
pub const ORIN_NX_8GB_DTB: &'static str = "tegra234-p3767-0001-p3768-0000-a0.dtb";
pub const ORIN_NX_16GB_DTS: &'static str = "tegra234-p3767-0000-p3768-0000-a0.dts";
pub const ORIN_NX_8GB_DTS: &'static str = "tegra234-p3767-0001-p3768-0000-a0.dts";

impl Jetson {
    pub fn new(bus: &str, dev: &str, vendor_number: &str, module_number: &str, instance_number: &str) -> Jetson {
        let mut ret = Jetson {
            bus: bus.to_string(),
            dev: dev.to_string(),
            vendor_number: vendor_number.to_string(),
            module_number: module_number.to_string(),
            module_name: String::new(),
            module_type: JetsonModuleType::None,
            instance_number: instance_number.to_string(),
            ip_v4: None,
            logger: None,
            status: FlashStatus::Wait,
        };

        if ret.module_number == "7323" {
            ret.set_module_type(JetsonModuleType::OrinNX16GB);
        } else if ret.module_number == "7423" {
            ret.set_module_type(JetsonModuleType::OrinNX8GB);
        } else if ret.module_number == "7e19" {
            ret.set_module_type(JetsonModuleType::XavierNX);
        } else if ret.module_number == "7523" {
            ret.set_module_type(JetsonModuleType::OrinNano8GB);
        } else if ret.module_number == "7623" {
            ret.set_module_type(JetsonModuleType::OrinNano4GB);
        } else if ["7023", "7223", "7019"].contains(&&ret.module_number[..]) {
            ret.set_module_type(JetsonModuleType::Unknown);
        } else {
            ret.set_module_type(JetsonModuleType::None);
        }
        
        let mut logger = Logger::new(&ret.module_name);
        logger.init();
        ret.logger = Some(logger);

        ret
    }

    pub fn set_module_type(&mut self, module_type: JetsonModuleType) {
        self.module_type = module_type;
        self.module_name = match self.module_type {
            JetsonModuleType::OrinNX16GB => {
                String::from("Jetson Orin NX 16GB")
            },
            JetsonModuleType::OrinNX8GB => {
                String::from("Jetson Orin NX 8GB")
            },
            JetsonModuleType::OrinNano8GB => {
                String::from("Jetson Orin Nano 8GB")
            },
            JetsonModuleType::OrinNano4GB => {
                String::from("Jetson Orin Nano 4GB")
            }
            JetsonModuleType::XavierNX => {
                String::from("Jetson Xavier NX")
            },
            _ => {
                String::from("Unknown USB Device")
            }
        };
    }

    pub fn to_string(&self) -> String {
        self.module_name.clone() + " (Bus " + &self.bus + " Device " + &self.dev + ": ID " + &self.vendor_number + ":" + &self.module_number + ")"
    }

    pub fn reset_flashing(&mut self) {
        self.status = FlashStatus::Failed;
    }

    pub fn set_flashing(&mut self) {
        self.status = FlashStatus::Flashing;
    }

    pub fn reset_flashed(&mut self) {
        self.status = FlashStatus::Wait;
    }

    pub fn set_flashed(&mut self) {
        self.status = FlashStatus::Finished;
    }

    pub fn is_flashing(&self) -> bool {
        self.status == FlashStatus::Flashing
    }

    pub fn is_flashed(&self) -> bool {
        self.status == FlashStatus::Finished
    }

    pub fn close_logger(&mut self) {
        self.logger.as_mut().unwrap().close();
    }

    pub fn open_logger(&mut self, name: &str) {
        self.logger = Some(Logger::new(name));
        self.logger.as_mut().unwrap().init();
    }

    pub fn create_new_publisher(&mut self) -> Sender<String> {
        self.logger.as_mut().unwrap().create_new_publisher()
    }
    
    pub fn get_logger_output(&mut self) -> &str {
        self.logger.as_mut().unwrap().output()
    }

    pub fn clear_logger_buffer(&mut self) {
        self.logger.as_mut().unwrap().clear();
    }
}
