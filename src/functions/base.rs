use crate::internal::config::get_packages;
use crate::internal::exec::*;
use crate::internal::files::append_file;
use crate::internal::*;
use log::warn;
use std::path::PathBuf;

pub fn install_base_packages(kernel: String) {
    std::fs::create_dir_all("/mnt/etc").unwrap();
    let kernel_to_install = if kernel.is_empty() {
        "linux"
    } else {
        match kernel.as_str() {
            "linux" => "linux",
            "linux-lts" => "linux-lts",
            "linux-zen" => "linux-zen",
            "linux-hardened" => "linux-hardened",
            _ => {
                warn!("Unknown kernel: {}, using default instead", kernel);
                "linux"
            }
        }
    };
    let kernel_headers_to_install = format!("{kernel_to_install}-headers");
    let pkg_list = get_packages();
    let mut pkgs: Vec<&str> = pkg_list.iter().map(|pkg| pkg.as_str()).collect();
    pkgs.insert(1, kernel_to_install);
    pkgs.insert(2, kernel_headers_to_install.as_str());

    install::install(pkgs);
    files::copy_file("/etc/pacman.conf", "/mnt/etc/pacman.conf");

    exec_eval(
        exec_chroot(
            "systemctl",
            vec![String::from("enable"), String::from("bluetooth")],
        ),
        "Enable bluetooth",
    );

    exec_eval(
        exec_chroot(
            "systemctl",
            vec![String::from("enable"), String::from("cups")],
        ),
        "Enable CUPS",
    );
}

pub fn genfstab() {
    exec_eval(
        exec(
            "bash",
            vec![
                String::from("-c"),
                String::from("genfstab -U /mnt >> /mnt/etc/fstab"),
            ],
        ),
        "Generate fstab",
    );
}

pub fn install_bootloader_grub_efi(efidir: PathBuf) {
    install::install(vec![
        "grub",
        "efibootmgr",
        "crystal-grub-theme",
        "os-prober",
        "crystal-branding",
    ]);
    let efidir = std::path::Path::new("/mnt").join(efidir);
    let efi_str = efidir.to_str().unwrap();
    if !std::path::Path::new(&format!("/mnt{efi_str}")).exists() {
        crash(format!("The efidir {efidir:?} doesn't exist"), 1);
    }
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![
                String::from("--target=x86_64-efi"),
                format!("--efi-directory={}", efi_str),
                String::from("--bootloader-id=crystal"),
                String::from("--removable"),
            ],
        ),
        "install grub as efi with --removable",
    );
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![
                String::from("--target=x86_64-efi"),
                format!("--efi-directory={}", efi_str),
                String::from("--bootloader-id=crystal"),
            ],
        ),
        "install grub as efi without --removable",
    );
    files_eval(
        append_file(
            "/mnt/etc/default/grub",
            "GRUB_THEME=\"/usr/share/grub/themes/crystal/theme.txt\"",
        ),
        "enable crystal grub theme",
    );
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

pub fn install_bootloader_grub_legacy(device: PathBuf) {
    install::install(vec![
        "grub",
        "crystal-grub-theme",
        "os-prober",
        "crystal-branding",
    ]);
    if !device.exists() {
        crash(format!("The device {device:?} does not exist"), 1);
    }
    let device = device.to_string_lossy().to_string();
    exec_eval(
        exec_chroot(
            "grub-install",
            vec![String::from("--target=i386-pc"), device],
        ),
        "install grub as legacy",
    );
    files_eval(
        append_file(
            "/mnt/etc/default/grub",
            "GRUB_THEME=\"/usr/share/grub/themes/crystal/theme.txt\"",
        ),
        "enable crystal grub theme",
    );
    exec_eval(
        exec_chroot(
            "grub-mkconfig",
            vec![String::from("-o"), String::from("/boot/grub/grub.cfg")],
        ),
        "create grub.cfg",
    );
}

pub fn install_bootloader_refind(efidir: PathBuf, default: bool, device: PathBuf) {
    install::install(vec![
        "refind",
        "efibootmgr",
        // "crystal-grub-theme",
        // "os-prober",
        // "crystal-branding",
    ]);
    let efidir = std::path::Path::new("/mnt").join(efidir);
    let efi_str = efidir.to_str().unwrap();
    if !std::path::Path::new(&format!("/mnt{efi_str}")).exists() {
        crash(format!("The efidir {efidir:?} doesn't exist"), 1);
    }

    // create esp dir for refind
    let refind_esp_path = match default {
        true => format!("/mnt{efi_str}/EFI/BOOT"),
        false => format!("/mnt{efi_str}/EFI/refind"),
    };
    files_eval(files::create_directory(&refind_esp_path), "create esp dir");

    // copy the efi binary
    match default {
        true => {
            files::copy_file(
                "/mnt/usr/share/refind/refind_x64.efi ",
                &format!("{refind_esp_path}/bootx64.efi"),
            );
        }
        false => {
            files::copy_file(
                "/mnt/usr/share/refind/refind_x64.efi ",
                &format!("{refind_esp_path}/refind_x64.efi"),
            );
        }
    };

    // create boot entry in the UEFI NVRAM, if installed as default then this isn't necessary
    if !default {
        exec_eval(
            exec_chroot(
                "efibootmgr",
                vec![
                    String::from("--create"),
                    String::from("--disk"),
                    String::from(device.to_string_lossy()),
                    String::from("--part"),
                    String::from("1"),
                    String::from("--loader"),
                    String::from("/EFI/refind/refind_x64.efi"),
                    String::from("--label"),
                    String::from("rEFInd Boot Manager"),
                    String::from("--unicode"),
                ],
            ),
            "creating uefi boot entry for refind",
        );
    };
}

pub fn setup_timeshift(bootloader: String) {
    let mut pkgs = vec!["timeshift", "timeshift-autosnap"];
    if bootloader.contains("grub") {
        pkgs.push("grub-btrfs");
    } // else if bootloader == "refind" {
      //     //
      // }
    install(pkgs);
    exec_eval(
        exec_chroot("timeshift", vec![String::from("--btrfs")]),
        "setup timeshift",
    )
}

pub fn install_homemgr() {
    install(vec!["nix"]);
}

pub fn install_flatpak() {
    install(vec!["flatpak"]);
    exec_eval(
        exec_chroot(
            "flatpak",
            vec![
                String::from("remote-add"),
                String::from("--if-not-exists"),
                String::from("flathub"),
                String::from("https://flathub.org/repo/flathub.flatpakrepo"),
            ],
        ),
        "Add flathub remote",
    )
}

pub fn install_zram() {
    install(vec!["zram-generator"]);
    files::create_file("/mnt/etc/systemd/zram-generator.conf");
    files_eval(
        files::append_file("/mnt/etc/systemd/zram-generator.conf", "[zram0]"),
        "Write zram-generator config",
    );
}
