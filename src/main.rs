mod args;
mod functions;
mod internal;
mod logging;

use crate::args::{BootloaderSubcommand, Command, Opt, UsersSubcommand};
use crate::functions::*;
use clap::Parser;

fn main() {
    human_panic::setup_panic!();
    let opt: Opt = Opt::parse();
    logging::init(opt.verbose);
    match opt.command {
        Command::Partition(args) => {
            let mut partitions = args.partitions;
            partition::partition(
                args.device,
                args.mode,
                args.efi,
                &mut partitions,
                args.unakite,
            );
        }
        Command::InstallBase(args) => {
            base::install_base_packages(args.kernel);
        }
        Command::GenFstab => {
            base::genfstab();
        }
        Command::SetupTimeshift { bootloader } => base::setup_timeshift(bootloader),
        Command::Bootloader { subcommand } => match subcommand {
            BootloaderSubcommand::GrubEfi { efidir } => {
                base::install_bootloader_grub_efi(efidir);
            }
            BootloaderSubcommand::GrubLegacy { device } => {
                base::install_bootloader_grub_legacy(device);
            }
            BootloaderSubcommand::Refind {
                efidir,
                default,
                device,
            } => {
                base::install_bootloader_refind(efidir, default, device);
            }
        },
        Command::Locale(args) => {
            locale::set_locale(args.locales.join(" "));
            locale::set_keyboard(&args.keyboard);
            locale::set_timezone(&args.timezone);
        }
        Command::Networking(args) => {
            if args.ipv6 {
                network::create_hosts();
                network::enable_ipv6()
            } else {
                network::create_hosts();
            }
            network::set_hostname(&args.hostname);
        }
        Command::Zram => {
            base::install_zram();
        }
        Command::Users { subcommand } => match subcommand {
            UsersSubcommand::NewUser(args) => {
                users::new_user(
                    &args.username,
                    args.hasroot,
                    &args.password,
                    true,
                    &args.shell,
                );
            }
            UsersSubcommand::RootPass { password } => {
                users::root_pass(&password);
            }
        },
        Command::Nix => {
            base::install_homemgr();
        }
        Command::Flatpak => {
            base::install_flatpak();
        }
        Command::Unakite(args) => {
            unakite::setup_unakite(
                &args.root,
                &args.oldroot,
                args.efi,
                &args.efidir,
                &args.bootdev,
            );
        }
        Command::Config { config } => {
            crate::internal::config::read_config(config);
        }
        Command::Desktops { desktop } => {
            desktops::install_desktop_setup(desktop);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::internal::config::get_packages;

    // use super::*;
    #[test]
    fn print_out() {
        let pkg_list = get_packages();
        for line in pkg_list {
            println!("{line}")
        }
        // assert_eq!(result, 4);
    }

    #[test]
    fn std_method() {
        let pkg_list = include_str!("../test.txt");
        // println!(pkg_list);

        for line in pkg_list.lines() {
            println!("{line}")
        }
    }
}
