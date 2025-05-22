use crate::functions::partition::umount;
use crate::internal::*;
use std::process::Command;

/// install packages on the new installation
pub fn install(pkgs: Vec<&str>) {
    exec_eval(
        Command::new("pacstrap").arg("/mnt").args(&pkgs).status(),
        format!("Installing packages: {}", pkgs.join(", ")).as_str(),
    );
    umount("/mnt/dev");
}
