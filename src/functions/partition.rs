use crate::args;
use crate::args::PartitionMode;
use crate::internal::exec::*;
use crate::internal::files::create_directory;
use crate::internal::*;
use std::path::{Path, PathBuf};

/*mkfs.bfs mkfs.cramfs mkfs.ext3  mkfs.fat mkfs.msdos  mkfs.xfs
mkfs.btrfs mkfs.ext2  mkfs.ext4  mkfs.minix mkfs.vfat mkfs.f2fs */

pub fn fmt_mount(mountpoint: &str, filesystem: &str, blockdevice: &str) {
    match filesystem {
        "vfat" => exec_eval(
            exec(
                "mkfs.vfat",
                vec![String::from("-F32"), String::from(blockdevice)],
            ),
            format!("Formatting {blockdevice} as vfat").as_str(),
        ),
        "bfs" => exec_eval(
            exec("mkfs.bfs", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as bfs").as_str(),
        ),
        "cramfs" => exec_eval(
            exec("mkfs.cramfs", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as cramfs").as_str(),
        ),
        "ext3" => exec_eval(
            exec("mkfs.ext3", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as ext3").as_str(),
        ),
        "fat" => exec_eval(
            exec("mkfs.fat", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as fat").as_str(),
        ),
        "msdos" => exec_eval(
            exec("mkfs.msdos", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as msdos").as_str(),
        ),
        "xfs" => exec_eval(
            exec("mkfs.xfs", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as xfs").as_str(),
        ),
        "btrfs" => exec_eval(
            exec(
                "mkfs.btrfs",
                vec![String::from("-f"), String::from(blockdevice)],
            ),
            format!("Formatting {blockdevice} as btrfs").as_str(),
        ),
        "ext2" => exec_eval(
            exec("mkfs.ext2", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as ext2").as_str(),
        ),
        "ext4" => exec_eval(
            exec("mkfs.ext4", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as ext4").as_str(),
        ),
        "minix" => exec_eval(
            exec("mkfs.minix", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as minix").as_str(),
        ),
        "f2fs" => exec_eval(
            exec("mkfs.f2fs", vec![String::from(blockdevice)]),
            format!("Formatting {blockdevice} as f2fs").as_str(),
        ),
        "don't format" => {
            log::debug!("Not formatting {}", blockdevice);
        }
        "noformat" => {
            log::debug!("Not formatting {}", blockdevice);
        }
        _ => {
            crash(
                format!("Unknown filesystem {filesystem}, used in partition {blockdevice}"),
                1,
            );
        }
    }
    exec_eval(
        exec("mkdir", vec![String::from("-p"), String::from(mountpoint)]),
        format!("Creating mountpoint {mountpoint} for {blockdevice}").as_str(),
    );
    mount(blockdevice, mountpoint, "");
}

pub fn partition(
    device: PathBuf,
    mode: PartitionMode,
    efi: bool,
    encrypted: bool,
    password: String,
    partitions: &mut Vec<args::Partition>,
) {
    println!("{:?}", mode);
    match mode {
        PartitionMode::Auto => {
            if !device.exists() {
                crash(format!("The device {device:?} doesn't exist"), 1);
            }
            log::debug!("automatically partitioning {device:?}");
            create_partitions(&device, efi);

            let part1: String; // This is probably a horrible way of doing this, but the borrow checker is annoying
            let part2: String;
            let part3: String;
            let mut partitions: Vec<&str> = if device.to_string_lossy().contains("nvme")
                || device.to_string_lossy().contains("mmcblk")
            {
                if efi {
                    part1 = format!("{}p1", device.to_string_lossy());
                    part2 = format!("{}p2", device.to_string_lossy());
                    part3 = format!("{}p3", device.to_string_lossy());
                    vec![part1.as_str(), part2.as_str(), part3.as_str()]
                } else {
                    part1 = format!("{}1", device.to_string_lossy());
                    part2 = format!("{}2", device.to_string_lossy());
                    vec![part1.as_str(), part2.as_str()]
                }
            } else {
                if efi {
                    part1 = format!("{}1", device.to_string_lossy());
                    part2 = format!("{}2", device.to_string_lossy());
                    part3 = format!("{}3", device.to_string_lossy());
                    vec![part1.as_str(), part2.as_str(), part3.as_str()]
                } else {
                    part1 = format!("{}1", device.to_string_lossy());
                    part2 = format!("{}2", device.to_string_lossy());
                    vec![part1.as_str(), part2.as_str()]
                }
            };
            let parts: Vec<String>;
            if encrypted {
                parts = encrypt_partition(partitions, &password, efi);
                partitions = parts.iter().map(|s| &**s).collect();
            }
            auto_format(efi, partitions);
            mount_disks(efi, encrypted);
        }
        PartitionMode::Manual => {
            log::debug!("Manual partitioning");
            partitions.sort_by(|a, b| a.mountpoint.len().cmp(&b.mountpoint.len()));
            for i in 0..partitions.len() {
                println!("{:?}", partitions);
                println!("{}", partitions.len());
                println!("{}", &partitions[i].mountpoint);
                println!("{}", &partitions[i].filesystem);
                println!("{}", &partitions[i].blockdevice);
                fmt_mount(
                    &partitions[i].mountpoint,
                    &partitions[i].filesystem,
                    &partitions[i].blockdevice,
                );
            }
        }
    }
}

fn create_partitions(device: &Path, efi: bool){
    let device = device.to_string_lossy().to_string();
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                String::from(&device),
                String::from("mklabel"),
                String::from(if efi { "gpt" } else { "msdos" }),
            ],
        ),
        format!(
            "Create {} label on {}",
            if efi { "gpt" } else { "msdos" },
            device
        )
        .as_str(),
    );
    if efi {
        exec_eval(
            exec(
                "parted",
                vec![
                    String::from("-s"),
                    String::from(&device),
                    String::from("mkpart"),
                    String::from("fat32"),
                    String::from("1MiB"),
                    String::from("125MiB"),
                ],
            ),
            "create EFI partition",
        );
        exec_eval(
            exec(
                "parted",
                vec![
                    String::from("-s"),
                    String::from(&device),
                    String::from("mkpart"),
                    String::from("primary"),
                    String::from("ext4"),
                    String::from("125MIB"),
                    String::from("637MIB"),
                ],
            ),
            "create boot partition",
        );
    } else {
        exec_eval(
            exec(
                "parted",
                vec![
                    String::from("-s"),
                    String::from(&device),
                    String::from("mkpart"),
                    String::from("primary"),
                    String::from("ext4"),
                    String::from("1MIB"),
                    String::from("512MIB"),
                ],
            ),
            "create bios boot partition",
        );
    }
    exec_eval(
        exec(
            "parted",
            vec![
                String::from("-s"),
                device,
                String::from("mkpart"),
                String::from("primary"),
                String::from("btrfs"),
                String::from(if efi { "637MiB" } else { "512MIB" } ),
                String::from("100%"),
            ],
        ),
        "create btrfs root partition",
    );
}

fn encrypt_partition(partitions: Vec<&str>, passphrase: &str, efi: bool) -> Vec<String> {
    exec_eval(
        exec(
            "bash",
            vec![
                String::from("-c"),
                format!(
                    "echo {} | cryptsetup luksFormat {}",
                    passphrase, if efi { partitions[2] } else { partitions[1] }
                ),
            ],
        ),
        format!("LUKS encrypt {}", if efi { partitions[2] } else { partitions[1] }).as_str(),
    );
    exec_eval(
        exec(
            "bash",
            vec![
                String::from("-c"),
                format!(
                    "echo {} | cryptsetup luksOpen {} root",
                    passphrase, if efi { partitions[2] } else { partitions[1] }
                ),
            ],
        ),
        format!("LUKS open {}", if efi { partitions[2] } else { partitions[1] }).as_str(),
    );
    exec_eval(
        exec(
            "cryptsetup",
            vec![
                String::from("close"),
                String::from("root"),
            ],
        ),
        format!("LUKS close {}", if efi { partitions[2] } else { partitions[1] }).as_str(),
    );
    exec_eval(
        exec(
            "bash",
            vec![
                String::from("-c"),
                format!(
                    "echo {} | cryptsetup luksOpen {} root",
                    passphrase, if efi { partitions[2] } else { partitions[1] }
                ),
            ],
        ),
        format!("LUKS open {}", if efi { partitions[2] } else { partitions[1] }).as_str(),
    );
    files::create_file("/tmp/encryption.sh");
    files_eval(
        files::append_file("/tmp/encryption.sh", "#!/bin/bash"),
        "Write encryption script",
    );
    files_eval(
        files::append_file("/tmp/encryption.sh", format!("UUID=$(lsblk -pdfo UUID {} | grep -v UUID)", if efi { partitions[2] } else { partitions[1] }).as_str()),
        "Write encryption script",
    );
    files_eval(
        files::append_file("/tmp/encryption.sh", "sed -i \"s/UUID=/UUID=${UUID}/g\" /mnt/etc/default/grub"),
        "Write encryption script",
    );
    if efi {
        return vec![
            partitions[0].to_string(),
            partitions[1].to_string(),
            "/dev/mapper/root".to_string(),
        ];
    } else {
        return vec![
            partitions[0].to_string(),
            "/dev/mapper/root".to_string(),
        ];
    }

}

fn auto_format(efi: bool, partitions: Vec<&str>) {
    println!("{:?}", partitions);
    if efi {
        exec_eval(
            exec(
                "mkfs.vfat",
                vec![
                    "-F32".to_string(),
                    "-n".to_string(),
                    "crystal-efi".to_string(),
                    partitions[0].to_string(),
                ],
            ),
            format!("format {} as fat32 with label crystal-efi", partitions[0]).as_str(),
        );
        exec_eval(
            exec(
                "mkfs.ext4",
                vec![
                    "-F".to_string(),
                    "-L".to_string(),
                    "crystal-boot".to_string(),
                    partitions[1].to_string(),
                ],
            ),
            format!("format {} as ext4 with label crystal-boot", partitions[1]).as_str(),
        );
    } else {
        exec_eval(
            exec(
                "mkfs.ext4",
                vec![
                    "-F".to_string(),
                    "-L".to_string(),
                    "crystal-boot".to_string(),
                    partitions[0].to_string(),
                ],
            ),
            format!("format {} as ext4 with label crystal-boot", partitions[0]).as_str(),
        )
    }
    exec_eval(
        exec(
            "mkfs.btrfs",
            vec![
                "-f".to_string(),
                "-L".to_string(),
                "crystal-root".to_string(),
                if efi { partitions[2].to_string() } else { partitions[1].to_string() },
            ],
        ),
        format!("format {} as btrfs with label crystal-root", if efi { partitions[2] } else { partitions[1] }).as_str(),
    );
}

fn mount_disks(efi: bool, encrypted: bool) {
    let root = if encrypted { "/dev/mapper/root" } else { "/dev/disk/by-label/crystal-root" };
    mount(root, "/mnt", "");
    exec_eval(
        exec_workdir(
            "btrfs",
            "/mnt",
            vec![
                String::from("subvolume"),
                String::from("create"),
                String::from("@"),
            ],
        ),
        "create btrfs subvolume @",
    );
    exec_eval(
        exec_workdir(
            "btrfs",
            "/mnt",
            vec![
                String::from("subvolume"),
                String::from("create"),
                String::from("@home"),
            ],
        ),
        "create btrfs subvolume @home",
    );
    umount("/mnt");
    mount(root, "/mnt", "subvol=@");
    files_eval(create_directory("/mnt/home"), "create directory /mnt/home");
    mount(
        root,
        "/mnt/home",
        "subvol=@home",
    );
    files_eval(create_directory("/mnt/boot"), "create directory /mnt/boot");
    mount("/dev/disk/by-label/crystal-boot", "/mnt/boot", "");
    if efi {
        files_eval(
            create_directory("/mnt/boot/efi"),
            "create directory /mnt/boot/efi",
        );
        mount("/dev/disk/by-label/crystal-efi", "/mnt/boot/efi", "");
    }
}

pub fn mount(partition: &str, mountpoint: &str, options: &str) {
    if !options.is_empty() {
        exec_eval(
            exec(
                "mount",
                vec![
                    String::from(partition),
                    String::from(mountpoint),
                    String::from("-o"),
                    String::from(options),
                ],
            ),
            format!(
                "mount {} with options {} at {}",
                partition, options, mountpoint
            )
            .as_str(),
        );
    } else {
        exec_eval(
            exec(
                "mount",
                vec![String::from(partition), String::from(mountpoint)],
            ),
            format!("mount {} with no options at {}", partition, mountpoint).as_str(),
        );
    }
}

pub fn umount(mountpoint: &str) {
    exec_eval(
        exec("umount", vec![String::from(mountpoint)]),
        format!("unmount {}", mountpoint).as_str(),
    );
}
