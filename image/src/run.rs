use std::process::Command;

pub fn run(image_path: &str) {
    let mut cmd = Command::new("qemu-system-i386");
    cmd.args(&["-drive", format!("format=raw,file={}", image_path).as_str()]);
    cmd.status().unwrap();
}
