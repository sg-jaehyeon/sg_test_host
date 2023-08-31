use std::sync::mpsc::Sender;
use std::fs::{self, OpenOptions};
use std::io::{Write, BufReader, BufRead};
use std::process::{Stdio, Command};
use std::thread::{self, JoinHandle};

use crate::jetson::*;
use crate::devicetree::{
    decompile::decompile_to_string,
    compile::compile_to_file,
    node::DtbNode,
};

pub fn check_env(tx: Sender<Signal>) -> bool {

    if check_test_env() && check_release_env() {
        tx.send(Signal::EnvironmentInstalled).unwrap();
        return true;
    }

    false
}

fn check_test_env() -> bool {
    // check env
    let ls_out = std::process::Command::new("ls")
                            .args([
                                "-d",
                                "test/Linux_for_Tegra",
                            ])
                            .stdout(Stdio::piped())
                            .stderr(Stdio::null())
                            .spawn()
                            .unwrap();

    let out = std::process::Command::new("wc")
                            .args([
                                "-l"
                            ])
                            .stdin(ls_out.stdout.unwrap())
                            .stderr(Stdio::null())
                            .output()
                            .unwrap()
                            .stdout;
                            

    let out = String::from_utf8(out).unwrap().trim().parse::<usize>();

    match out {
        Ok(0) => {
            return false;
        },
        Ok(1) => {
            return true;
        },
        _ => {
        panic!("Please check workspace");
        }
    }
}

fn check_release_env() -> bool {
    // check env
    let ls_out = std::process::Command::new("ls")
                            .args([
                                "-d",
                                "release/Linux_for_Tegra",
                            ])
                            .stdout(Stdio::piped())
                            .stderr(Stdio::null())
                            .spawn()
                            .unwrap();

    let out = std::process::Command::new("wc")
                            .args([
                                "-l"
                            ])
                            .stdin(ls_out.stdout.unwrap())
                            .stderr(Stdio::null())
                            .output()
                            .unwrap()
                            .stdout;
                            

    let out = String::from_utf8(out).unwrap().trim().parse::<usize>();

    match out {
        Ok(0) => {
            return false;
        },
        Ok(1) => {
            return true;
        },
        _ => {
        panic!("Please check workspace");
        }
    }
}

pub fn setup_workspace(tx: Sender<Signal>, path: &str) -> JoinHandle<()> {
    let path = path.to_string();
    let handle = thread::spawn(move || {
        let path = &path[..];
        tx.send(Signal::Message(String::from("[1/7] Download Jetson Linux...\n"))).unwrap();
        let _ = download_jetson_linux(tx.clone(), &path);
        tx.send(Signal::Message(String::from("[2/7] Patch device tree...\n"))).unwrap();
        let _ = patch_device_tree(tx.clone(), &path);
        tx.send(Signal::Message(String::from("[3/7] Apply binaries...\n"))).unwrap();
        let _ = apply_binaries(tx.clone());
        tx.send(Signal::Message(String::from("[4/7] Create default user...\n"))).unwrap();
        let _ = create_default_user(tx.clone());
        tx.send(Signal::Message(String::from("[5/7] Install startup programs...\n"))).unwrap();
        let _ = install_test_client(tx.clone());
        tx.send(Signal::Message(String::from("[6/7] Generating massflash package for test environment...\n"))).unwrap();
        tx.send(Signal::Message(String::from("It may take a very long time.\n"))).unwrap();
        let _  = generate_massflash_package(tx.clone());
        tx.send(Signal::Message(String::from("Workspace setup has finished\n"))).unwrap();
        tx.send(Signal::EnvironmentInstalled).unwrap();
    });

    handle
}

fn download_jetson_linux(tx: Sender<Signal>, path: &str) -> Result<(), Box<dyn std::error::Error>> {

    let workspace = path.clone().to_string();
    let jetson_linux = String::from(&workspace) + "/jetson_linux_r35.3.1_aarch64.tbz2";
    let rootfs = String::from(&workspace) + "/tegra_linux_sample-root-filesystem_r35.3.1_aarch64.tbz2";
    let test_rootfs_target= String::from(&workspace) + "/test/Linux_for_Tegra/rootfs/";
    let sources = String::from(&workspace) + "/public_sources.tbz2";
    let release_rootfs_target = String::from(&workspace) + "/release/Linux_for_Tegra/rootfs/";

    let mut child = std::process::Command::new("rm")
                        .args([
                            "-rf",
                            &jetson_linux,
                            &sources,
                            &rootfs,
                            "wget.log",
                            "Linux_for_Tegra",
                            "test",
                            "release",
                        ])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn()
                        .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    tx.send(Signal::Message(String::from("Cleaning workspace...\n"))).unwrap();
    let _ = std::process::Command::new("mkdir")
                        .args([
                            "test",
                            "release",
                        ])
                        .output()
                        .unwrap();

    tx.send(Signal::Message(String::from("Downloading files from server...\n"))).unwrap();
    let mut child = Command::new("wget")
                    .args([
                        "https://developer.nvidia.com/downloads/embedded/l4t/r35_release_v3.1/release/jetson_linux_r35.3.1_aarch64.tbz2/",
                        "-O",
                        &jetson_linux,
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    let mut child = Command::new("wget")
                    .args([
                        "https://developer.nvidia.com/downloads/embedded/l4t/r35_release_v3.1/release/tegra_linux_sample-root-filesystem_r35.3.1_aarch64.tbz2/",
                        "-O",
                        &rootfs,
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    let mut child = Command::new("wget")
                    .args([
                        "https://developer.nvidia.com/downloads/embedded/l4t/r35_release_v3.1/sources/public_sources.tbz2/",
                        "-O",
                        &sources,
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    let jetson_linux = String::from(&workspace) + "/jetson_linux_r35.3.1_aarch64.tbz2";
    let rootfs = String::from(&workspace) + "/tegra_linux_sample-root-filesystem_r35.3.1_aarch64.tbz2";
    let sources = String::from(&workspace) + "/public_sources.tbz2";

    tx.send(Signal::Message(String::from("Extracting files for test environment...\n"))).unwrap();
    let mut child = std::process::Command::new("tar")
                    .args([
                        "xf",
                        &jetson_linux,
                        "-C",
                        "./test",
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    let mut child = std::process::Command::new("tar")
                    .args([
                        "xf",
                        &sources,
                        "-C",
                        "./test",
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    let mut child = std::process::Command::new("tar")
                    .args([
                        "xpf",
                        &rootfs,
                        "-C",
                        &test_rootfs_target,
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    tx.send(Signal::Message(String::from("Extracting files for release environment...\n"))).unwrap();
    let mut child = std::process::Command::new("tar")
                .args([
                    "xf",
                    &jetson_linux,
                    "-C",
                    "./release",
                ])
                .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    let mut child = std::process::Command::new("tar")
                    .args([
                        "xf",
                        &sources,
                        "-C",
                        "./release",
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    let mut child = std::process::Command::new("tar")
                    .args([
                        "xpf",
                        &rootfs,
                        "-C",
                        &release_rootfs_target,
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    let jetson_linux = String::from(&workspace) + "/jetson_linux_r35.3.1_aarch64.tbz2";
    let rootfs = String::from(&workspace) + "/tegra_linux_sample-root-filesystem_r35.3.1_aarch64.tbz2";
    let sources = String::from(&workspace) + "/public_sources.tbz2";
    tx.send(Signal::Message(String::from("Removing archive files...\n"))).unwrap();
    let mut child = std::process::Command::new("rm")
                        .args([
                            &jetson_linux,
                            &sources,
                            &rootfs,
                        ])
                        .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    reader.lines().filter_map(|line| line.ok()).for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); } );
    child.wait().unwrap();

    Ok(())
}

fn patch_device_tree(tx: Sender<Signal>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    tx.send(Signal::Message(String::from("Patching") + XAVIER_NX_DTB + "\n")).unwrap();
    patch_device_tree_xavier_nx(tx.clone(), path)?;
    tx.send(Signal::Message(String::from("Patching") + ORIN_NX_16GB_DTB + "\n")).unwrap();
    patch_device_tree_orin_nx_16gb(tx.clone(), path)?;
    tx.send(Signal::Message(String::from("Patching") + ORIN_NX_8GB_DTB + "\n")).unwrap();
    patch_device_tree_orin_nx_8gb(tx.clone(), path)?;
    Ok(())
}

fn patch_device_tree_xavier_nx(tx: Sender<Signal>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dtb = path.to_string() + "/test/Linux_for_Tegra/kernel/dtb/" + XAVIER_NX_DTB;
    let dts = path.to_string() + "/test/Linux_for_Tegra/kernel/dtb/" + XAVIER_NX_DTS;

    let mut root_node = DtbNode::new(decompile_to_string(tx.clone(), &dtb)?);

    // sdcard slot
    match root_node.find_childnode("sdhci@3440000") {
        Some(sdhci) => {
            let status = sdhci.find_property("status").unwrap();
            status.value = Some("\"okay\"".to_string());
        },
        None => {
            panic!("device tree corrupted");
        }
    }

    // camera
    let cam_i2c0 = root_node
                                                .find_childnode("cam_i2cmux").unwrap()
                                                .find_childnode("i2c@0").unwrap();

    let rbpcv3_imx477_a_1a = cam_i2c0
                                    .find_childnode("rbpcv3_imx477_a@1a").unwrap();
    
    rbpcv3_imx477_a_1a.find_property("status").unwrap()
                        .value = Some("\"okay\"".to_string());

    // rbpcv3_imx477_a_1a.find_childnode("mode0").unwrap()
    //                     .find_property("tegra_sinterface").unwrap()
    //                     .value = Some("\"serial_a\"".to_string());

    // rbpcv3_imx477_a_1a.find_childnode("mode1").unwrap()
    //                     .find_property("tegra_sinterface").unwrap()
    //                     .value = Some("\"serial_a\"".to_string());
    
    // rbpcv3_imx477_a_1a.find_childnode("ports").unwrap()
    //                     .find_childnode("port@0").unwrap()
    //                     .find_childnode("endpoint").unwrap()
    //                     .find_property("port-index").unwrap()
    //                     .value = Some("<0x00>".to_string());

    // let rbpcv2_imx219_a_10 = cam_i2c0.find_childnode("rbpcv2_imx219_a@10").unwrap();

    // rbpcv2_imx219_a_10.find_childnode("mode0").unwrap()
    //                 .find_property("tegra_sinterface").unwrap()
    //                 .value = Some("\"serial_a\"".to_string());

    // rbpcv2_imx219_a_10.find_childnode("mode1").unwrap()
    //                 .find_property("tegra_sinterface").unwrap()
    //                 .value = Some("\"serial_a\"".to_string());

    // rbpcv2_imx219_a_10.find_childnode("mode2").unwrap()
    //                 .find_property("tegra_sinterface").unwrap()
    //                 .value = Some("\"serial_a\"".to_string());

    // rbpcv2_imx219_a_10.find_childnode("mode3").unwrap()
    //                 .find_property("tegra_sinterface").unwrap()
    //                 .value = Some("\"serial_a\"".to_string());

    // rbpcv2_imx219_a_10.find_childnode("mode4").unwrap()
    //                 .find_property("tegra_sinterface").unwrap()
    //                 .value = Some("\"serial_a\"".to_string());

    // rbpcv2_imx219_a_10.find_childnode("ports").unwrap()
    //                 .find_childnode("port@0").unwrap()
    //                 .find_childnode("endpoint").unwrap()
    //                 .find_property("port-index").unwrap()
    //                 .value = Some("<0x00>".to_string());

    let cam_i2c1 = root_node
                    .find_childnode("cam_i2cmux").unwrap()
                    .find_childnode("i2c@1").unwrap();

    let rbpcv3_imx477_c_1a = cam_i2c1
                    .find_childnode("rbpcv3_imx477_c@1a").unwrap();

    rbpcv3_imx477_c_1a.find_property("status").unwrap()
                    .value = Some("\"okay\"".to_string());

    let patched_string = root_node.stringify(0);
    let mut patched_dts = OpenOptions::new()
                                    .write(true)
                                    .truncate(true)
                                    .create(true)
                                    .open(&dts)?;
    patched_dts.write_all(patched_string.as_bytes())?;
    compile_to_file(&dts, &dtb)?;

    Ok(())
}

fn patch_device_tree_orin_nx_16gb(tx: Sender<Signal>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dtb = path.to_string() + "/test/Linux_for_Tegra/kernel/dtb/" + ORIN_NX_16GB_DTB;
    let dts = path.to_string() + "/test/Linux_for_Tegra/kernel/dtb/" + ORIN_NX_16GB_DTS;

    let mut root_node = DtbNode::new(decompile_to_string(tx.clone(), &dtb)?);

    // camera
    let cam_i2c0 = root_node
                                                .find_childnode("cam_i2cmux").unwrap()
                                                .find_childnode("i2c@0").unwrap();

    let rbpcv3_imx477_a_1a = cam_i2c0
                                    .find_childnode("rbpcv3_imx477_a@1a").unwrap();
    
    rbpcv3_imx477_a_1a.find_property("status").unwrap()
                        .value = Some("\"okay\"".to_string());

    rbpcv3_imx477_a_1a.find_childnode("mode0").unwrap()
                        .find_property("tegra_sinterface").unwrap()
                        .value = Some("\"serial_a\"".to_string());

    rbpcv3_imx477_a_1a.find_childnode("mode1").unwrap()
                        .find_property("tegra_sinterface").unwrap()
                        .value = Some("\"serial_a\"".to_string());
    
    rbpcv3_imx477_a_1a.find_childnode("ports").unwrap()
                        .find_childnode("port@0").unwrap()
                        .find_childnode("endpoint").unwrap()
                        .find_property("port-index").unwrap()
                        .value = Some("<0x00>".to_string());

    let rbpcv2_imx219_a_10 = cam_i2c0.find_childnode("rbpcv2_imx219_a@10").unwrap();

    rbpcv2_imx219_a_10.find_childnode("mode0").unwrap()
                    .find_property("tegra_sinterface").unwrap()
                    .value = Some("\"serial_a\"".to_string());

    rbpcv2_imx219_a_10.find_childnode("mode1").unwrap()
                    .find_property("tegra_sinterface").unwrap()
                    .value = Some("\"serial_a\"".to_string());

    rbpcv2_imx219_a_10.find_childnode("mode2").unwrap()
                    .find_property("tegra_sinterface").unwrap()
                    .value = Some("\"serial_a\"".to_string());

    rbpcv2_imx219_a_10.find_childnode("mode3").unwrap()
                    .find_property("tegra_sinterface").unwrap()
                    .value = Some("\"serial_a\"".to_string());

    rbpcv2_imx219_a_10.find_childnode("mode4").unwrap()
                    .find_property("tegra_sinterface").unwrap()
                    .value = Some("\"serial_a\"".to_string());

    rbpcv2_imx219_a_10.find_childnode("ports").unwrap()
                    .find_childnode("port@0").unwrap()
                    .find_childnode("endpoint").unwrap()
                    .find_property("port-index").unwrap()
                    .value = Some("<0x00>".to_string());

    let cam_i2c1 = root_node
                    .find_childnode("cam_i2cmux").unwrap()
                    .find_childnode("i2c@1").unwrap();

    let rbpcv3_imx477_c_1a = cam_i2c1
                    .find_childnode("rbpcv3_imx477_c@1a").unwrap();

    rbpcv3_imx477_c_1a.find_property("status").unwrap()
                    .value = Some("\"okay\"".to_string());

    let patched_string = root_node.stringify(0);
    let mut patched_dts = OpenOptions::new()
                                    .write(true)
                                    .truncate(true)
                                    .create(true)
                                    .open(&dts)?;
    patched_dts.write_all(patched_string.as_bytes())?;
    compile_to_file(&dts, &dtb)?;

    Ok(())
}

fn patch_device_tree_orin_nx_8gb(tx: Sender<Signal>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dtb = path.to_string() + "/test/Linux_for_Tegra/kernel/dtb/" + ORIN_NX_8GB_DTB;
    let dts = path.to_string() + "/test/Linux_for_Tegra/kernel/dtb/" + ORIN_NX_8GB_DTS;

    let dts_content = decompile_to_string(tx.clone(), &dtb)?;

    let mut root_node = DtbNode::new(dts_content);

    // camera
    let cam_i2c0 = root_node
                                                .find_childnode("cam_i2cmux").unwrap()
                                                .find_childnode("i2c@0").unwrap();

    let rbpcv3_imx477_a_1a = cam_i2c0
                                    .find_childnode("rbpcv3_imx477_a@1a").unwrap();
    
    rbpcv3_imx477_a_1a.find_property("status").unwrap()
                        .value = Some("\"okay\"".to_string());

    rbpcv3_imx477_a_1a.find_childnode("mode0").unwrap()
                        .find_property("tegra_sinterface").unwrap()
                        .value = Some("\"serial_a\"".to_string());

    rbpcv3_imx477_a_1a.find_childnode("mode1").unwrap()
                        .find_property("tegra_sinterface").unwrap()
                        .value = Some("\"serial_a\"".to_string());
    
    rbpcv3_imx477_a_1a.find_childnode("ports").unwrap()
                        .find_childnode("port@0").unwrap()
                        .find_childnode("endpoint").unwrap()
                        .find_property("port-index").unwrap()
                        .value = Some("<0x00>".to_string());

    let rbpcv2_imx219_a_10 = cam_i2c0.find_childnode("rbpcv2_imx219_a@10").unwrap();

    rbpcv2_imx219_a_10.find_childnode("mode0").unwrap()
                    .find_property("tegra_sinterface").unwrap()
                    .value = Some("\"serial_a\"".to_string());

    rbpcv2_imx219_a_10.find_childnode("mode1").unwrap()
                    .find_property("tegra_sinterface").unwrap()
                    .value = Some("\"serial_a\"".to_string());

    rbpcv2_imx219_a_10.find_childnode("mode2").unwrap()
                    .find_property("tegra_sinterface").unwrap()
                    .value = Some("\"serial_a\"".to_string());

    rbpcv2_imx219_a_10.find_childnode("mode3").unwrap()
                    .find_property("tegra_sinterface").unwrap()
                    .value = Some("\"serial_a\"".to_string());

    rbpcv2_imx219_a_10.find_childnode("mode4").unwrap()
                    .find_property("tegra_sinterface").unwrap()
                    .value = Some("\"serial_a\"".to_string());

    rbpcv2_imx219_a_10.find_childnode("ports").unwrap()
                    .find_childnode("port@0").unwrap()
                    .find_childnode("endpoint").unwrap()
                    .find_property("port-index").unwrap()
                    .value = Some("<0x00>".to_string());

    let cam_i2c1 = root_node
                    .find_childnode("cam_i2cmux").unwrap()
                    .find_childnode("i2c@1").unwrap();

    let rbpcv3_imx477_c_1a = cam_i2c1
                    .find_childnode("rbpcv3_imx477_c@1a").unwrap();

    rbpcv3_imx477_c_1a.find_property("status").unwrap()
                    .value = Some("\"okay\"".to_string());

    let patched_string = root_node.stringify(0);
    let mut patched_dts = OpenOptions::new()
                                    .write(true)
                                    .truncate(true)
                                    .create(true)
                                    .open(&dts)?;
    patched_dts.write_all(patched_string.as_bytes())?;
    compile_to_file(&dts, &dtb)?;

    Ok(())
}

fn apply_binaries(tx: Sender<Signal>) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = std::process::Command::new("./test/Linux_for_Tegra/apply_binaries.sh")
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::null())
                                    .spawn()
                                    .unwrap();

    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);

    reader.lines()
        .filter_map(|line| line.ok())
        .for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); });

    child.wait().unwrap();

    let mut child = std::process::Command::new("./release/Linux_for_Tegra/apply_binaries.sh")
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::null())
                                    .spawn()
                                    .unwrap();

    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);

    reader.lines()
        .filter_map(|line| line.ok())
        .for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap(); });

    child.wait().unwrap();

    Ok(())
}

fn create_default_user(tx: Sender<Signal>) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("./test/Linux_for_Tegra/tools/l4t_create_default_user.sh")
                                        .args([
                                            "-u",
                                            "jetson",
                                            "-p",
                                            "jetson",
                                            "-a",
                                            "--accept-license",
                                        ])
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::null())
                                        .spawn()
                                        .unwrap();

    let output = child.stdout.take().unwrap();
    let reader = BufReader::new(output);
    
    reader.lines()
        .filter_map(|line| line.ok())
        .for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap() });

    child.wait().unwrap();

    Ok(())
}

fn install_test_client(_tx: Sender<Signal>) -> Result<(), Box<dyn std::error::Error>> {
    // copy script to launch test process
    let _ = fs::copy("./client/launch_test.sh", "./test/Linux_for_Tegra/rootfs/launch_test.sh");

    // copy binary
    let _ = fs::copy("./client/sg_test_client", "./test/Linux_for_Tegra/rootfs/sg_test_client");
    
    Ok(())
}

fn generate_massflash_package(tx: Sender<Signal>) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("./tools/kernel_flash/l4t_initrd_flash.sh")
                                        .current_dir("./test/Linux_for_Tegra/")
                                        .args([
                                            "--no-flash",
                                            "--external-device",
                                            "nvme0n1",
                                            "-c",
                                            "tools/kernel_flash/flash_l4t_external.xml",
                                            "-p",
                                            "-c bootloader/t186ref/cfg/flash_t234_qspi.xml",
                                            "--network",
                                            "usb0",
                                            "--showlogs",
                                            "--massflash",
                                            "jetson-orin-nano-devkit",
                                            "internal",
                                        ])
                                        .stdout(Stdio::null())
                                        .stderr(Stdio::null())
                                        .spawn()
                                        .unwrap();

    // let output = child.stdout.take().unwrap();
    // let reader = BufReader::new(output);
    
    // reader.lines()
    //     .filter_map(|line| line.ok())
    //     .for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap() });

    child.wait().unwrap();

    tx.send(Signal::Message(String::from("[7/7] Generating massflash package for release environment...\n"))).unwrap();
    tx.send(Signal::Message(String::from("It may take a very long time.\n"))).unwrap();
    let mut child = Command::new("./tools/kernel_flash/l4t_initrd_flash.sh")
                                        .current_dir("./release/Linux_for_Tegra")
                                        .args([
                                            "--no-flash",
                                            "--external-device",
                                            "nvme0n1",
                                            "-c",
                                            "tools/kernel_flash/flash_l4t_external.xml",
                                            "-p",
                                            "-c bootloader/t186ref/cfg/flash_t234_qspi.xml",
                                            "--network",
                                            "usb0",
                                            "--showlogs",
                                            "--massflash",
                                            "jetson-orin-nano-devkit",
                                            "internal",
                                        ])
                                        .stdout(Stdio::null())
                                        .stderr(Stdio::null())
                                        .spawn()
                                        .unwrap();

    // let output = child.stdout.take().unwrap();
    // let reader = BufReader::new(output);
    
    // reader.lines()
    //     .filter_map(|line| line.ok())
    //     .for_each(|line| { tx.send(Signal::Message(line + "\n")).unwrap() });

    child.wait().unwrap();

    Ok(())
}