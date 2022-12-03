use std::env;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let ssh_config = get_ssh_config_path();
    let ssh_config = fs::read_to_string(ssh_config).unwrap();
    let defined_hosts: Vec<_> = ssh_config
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("Host "))
        .collect();
    let vm = env::args()
        .skip(1)
        .find(|arg| defined_hosts.contains(&arg.deref()));
    if let Some(vm) = vm {
        #[rustfmt::skip]
        let _start_vm = Command::new("multipass")
            .args(["start", &vm])
            .spawn().unwrap()
            .wait().unwrap();
    }
    #[rustfmt::skip]
    let _ssh_cmd = Command::new("ssh")
        .args(env::args().skip(1))
        .spawn().unwrap()
        .wait().unwrap();
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
