use std::{
    env,
    error::Error,
    fs::File,
    io::{self, Seek},
    path::{Path, PathBuf},
    process::Command,
};

use mbrman::{MBRPartitionEntry, BOOT_ACTIVE, BOOT_INACTIVE, CHS, MBR};

fn main() {
    // Get the output directory from OUT_DIR, default to 'bin'.
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| "bin".to_string());

    let stage_1_path = build(&out_dir, "bootloader/stage-1", "stage-1");
    let stage_2_path = build(&out_dir, "bootloader/stage-2", "stage-2");
    let kernel_path = build(&out_dir, "kernel/", "kernel");

    build_image(
        &out_dir,
        &stage_1_path.to_str().unwrap(),
        &stage_2_path.to_str().unwrap(),
        &kernel_path.to_str().unwrap(),
    )
    .expect("Failed to build disk image");
}

// build builds the package at the given path.
// Rust builds to ELF files by default. After building, this function
// calls elf_to_bin to convert the ELF file to a binary file.
// e.g. cargo build --config bootloader/stage-1/.cargo/config.toml \
//                  --release -p stage-1 \
//                  -Zunstable-options --out-dir bin
// @param path: The path to the package.
// @param package_name: The name of the package to build.
// @return: The path to the binary file.
fn build(out_dir: &str, path: &str, package_name: &str) -> PathBuf {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cmd = Command::new(cargo);

    // Pass the path to the .cargo/config.toml file if it exists.
    // All no_std crates will need this because this build script
    // does not know build specifics.
    let cargo_config_path = Path::new(path).join(".cargo/config.toml");
    if cargo_config_path.exists() {
        cmd.arg("--config").arg(cargo_config_path);
    }

    cmd.arg("build")
        .arg("--release")
        .arg("-p")
        .arg(&package_name)
        .arg("-Zunstable-options") // Needed for --out-dir
        .arg("--out-dir")
        .arg(&out_dir);

    cmd.status().expect(&format!(
        "Failed to execute cargo build on {}",
        package_name
    ));

    let elf_path = Path::new(&out_dir).join(&package_name);
    elf_to_bin(&elf_path)
}

// elf_to_bin converts an ELF file to a binary file using llvm-objcopy.
// e.g. llvm-objcopy -I elf32-i386 -O binary stage-1 stage-1.bin
// @param elf_path: The path to the ELF file.
// @return: The path to the binary file.
fn elf_to_bin(elf_path: &PathBuf) -> PathBuf {
    let bin_path = elf_path.with_extension("bin");

    let llvm_tools = llvm_tools::LlvmTools::new().expect("Failed to find llvm-tools");
    let objcopy = llvm_tools
        .tool(&llvm_tools::exe("llvm-objcopy"))
        .expect("Failed to find llvm-objcopy");

    let mut cmd = Command::new(objcopy);
    cmd.arg("-I")
        .arg("elf32-i386")
        .arg("-O")
        .arg("binary")
        .arg(&elf_path)
        .arg(&bin_path);

    cmd.status().expect("Failed to execute objcopy");

    bin_path
}

// build_image builds the disk image.
// Creates a disk image with the following layout:
// 0x0000 - 0x01FF: MBR
// 0x0200 - 0x0FFF: Stage 2
// @param out_dir: The output directory.
// @param stage_1_path: The path to the stage 1 binary.
// @param second_stage_path: The path to the stage 2 binary.
fn build_image(
    out_dir: &str,
    stage_1_path: &str,
    second_stage_path: &str,
    kernel_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut stage_1 = File::open(stage_1_path)?;
    let mut second_stage = File::open(second_stage_path)?;
    let mut kernel = File::open(kernel_path)?;

    let mut mbr = MBR::read_from(&mut stage_1, 512)?;

    // Add stages and kernel to the partition table.
    // Partitions are 1-indexed.
    mbr[1] = MBRPartitionEntry {
        boot: BOOT_ACTIVE,
        starting_lba: 1,
        sectors: (second_stage.metadata()?.len() - 1) as u32 / 512 + 1,
        sys: 0x02,
        first_chs: CHS::empty(),
        last_chs: CHS::empty(),
    };

    mbr[2] = MBRPartitionEntry {
        boot: BOOT_ACTIVE,
        starting_lba: 1 + mbr[1].sectors,
        sectors: (kernel.metadata()?.len() - 1) as u32 / 512 + 1,
        sys: 0x83,
        first_chs: CHS::empty(),
        last_chs: CHS::empty(),
    };

    let mut disk_image = File::create(out_dir.to_owned() + "/disk_image.bin")?;

    mbr.write_into(&mut disk_image)?;
    assert_eq!(disk_image.stream_position()?, 512);

    io::copy(&mut second_stage, &mut disk_image)?;
    assert_eq!(
        disk_image.stream_position()?,
        512 + second_stage.metadata()?.len()
    );

    io::copy(&mut kernel, &mut disk_image)?;
    assert_eq!(
        disk_image.stream_position()?,
        512 + second_stage.metadata()?.len() + kernel.metadata()?.len()
    );

    Ok(())
}
