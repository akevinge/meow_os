mod ascii;
mod image;

use std::{path::Path, process::Command};

use clap::{Args, Parser, Subcommand};
use tempfile::NamedTempFile;

type DynErr = Box<dyn std::error::Error>;

#[derive(Args)]
struct BuildOptions {
    #[arg(short = 'p', long = "image-path", default_value = "target/disk.img")]
    image_path: String,
    #[arg(short = 't', long = "target", default_value = "x86_64-unknown-uefi")]
    target: String,
}

#[derive(Args)]
struct RunOptions {
    #[arg(short = 'p', long = "image-path", default_value = "target/disk.img")]
    image_path: String,
}

#[derive(Subcommand)]
enum XTaskCmd {
    Build(BuildOptions),
    Run(RunOptions),
}

#[derive(Parser)]
struct CLI {
    #[command(subcommand)]
    command: XTaskCmd,
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), DynErr> {
    let cli = CLI::parse();

    match &cli.command {
        XTaskCmd::Build(opts) => build(opts)?,
        XTaskCmd::Run(opts) => run(opts)?,
    }

    Ok(())
}

fn build(opts: &BuildOptions) -> Result<(), DynErr> {
    println!("{}", ascii::BUILDING);
    println!("{}", ascii::SEP);
    println!("{}", ascii::SEP);
    println!("{}\n", ascii::SEP);

    built_bootloader(opts)?;
    build_image(opts)?;

    Ok(())
}

/// Builds the bootloader.
fn built_bootloader(opts: &BuildOptions) -> Result<(), DynErr> {
    println!("{}", ascii::UEFI);

    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    cmd.args(&["--package", "bootloader"]);
    cmd.args(&["--target", &opts.target]);
    cmd.status()?;

    Ok(())
}

fn build_image(opts: &BuildOptions) -> Result<(), DynErr> {
    // Create a FAT filesystem image.
    // FAT file is temporary because it will be copied into the disk image.
    let fat_image = NamedTempFile::new()?;
    image::create_fat_fs(
        &fat_image,
        vec![(
            "target/x86_64-unknown-uefi/debug/bootloader.efi",
            "efi/boot/bootx64.efi",
        )],
    )
    .map_err(|e| {
        eprintln!("Failed to create FAT filesystem image: {}", e);
        e
    })?;
    // Create a GPT disk image.
    image::create_gpt_disk(&Path::new(&opts.image_path), fat_image.path())?;

    Ok(())
}

fn run(opts: &RunOptions) -> Result<(), DynErr> {
    println!("{}", ascii::RUNNING);
    println!("{}", ascii::SEP);
    println!("{}", ascii::SEP);
    println!("{}\n", ascii::SEP);

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.args(&["-bios", "/usr/share/ovmf/OVMF.fd"]);
    cmd.args(&["-drive", &format!("format=raw,file={}", &opts.image_path)]);
    cmd.status()?;

    Ok(())
}