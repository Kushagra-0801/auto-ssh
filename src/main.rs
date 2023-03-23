use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

#[cfg(windows)]
const NULL_FILE: &str = "NUL";
#[cfg(not(windows))]
const NULL_FILE: &str = "/dev/null";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VmProvider {
    Multipass,
    VirtualBox,
}

use VmProvider::{Multipass, VirtualBox};

const CONFIG: &[(&str, VmProvider)] = &[("work", Multipass), ("devops", VirtualBox)];

fn main() {
    let vm = env::args()
        .skip(1)
        .find_map(|arg| CONFIG.iter().find(|(name, _)| name == &arg));
    let Some(vm) = vm else { return spawn_ssh(); };
    match vm {
        (vm, Multipass) => {
            start_multipass_vm(vm);
            spawn_ssh();
        }
        (vm, VirtualBox) => {
            start_virtualbox_vm(vm);
            spawn_ssh();
        }
    }
}

fn start_virtualbox_vm(vm: &str) {
    Command::new("VBoxManage")
        .args(["startvm", vm, "--type", "headless"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn start_multipass_vm(vm: &str) {
    let vm_ip = Command::new("multipass")
        .args(["exec", vm, "--", ".local/bin/get-ip", "192"])
        .output()
        .unwrap();
    let ip = String::from_utf8(vm_ip.stdout).unwrap().trim().to_owned();
    let conf_contents = indoc::formatdoc!(
        "
        Host {vm}
            Hostname {ip}
            User ubuntu
            StrictHostKeyChecking no
            UserKnownHostsFile {NULL_FILE}
            GlobalKnownHostsFile={NULL_FILE}
        "
    );
    let conf_path = get_ssh_config_path().with_file_name("multipassvm.conf");
    fs::write(conf_path, conf_contents).unwrap();
}

fn spawn_ssh() {
    Command::new("ssh")
        .args(env::args().skip(1))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn get_ssh_config_path() -> PathBuf {
    let mut ssh_config = if cfg!(target_os = "windows") {
        if let Ok(base_path) = env::var("USERPROFILE") {
            PathBuf::from(base_path)
        } else {
            let mut drive = env::var("HOMEDRIVE").expect("HOMEDRIVE not defined");
            let path = env::var("HOMEPATH").expect("HOMEPATH not defined");
            drive.push_str(&path);
            PathBuf::from(drive)
        }
    } else {
        PathBuf::from(env::var("HOME").expect("HOME not defined"))
    };
    ssh_config.push(".ssh/config");
    assert!(
        ssh_config.is_file(),
        "{ssh_config:?} is not the correct path to the ssh config"
    );
    ssh_config
}
