use std::{
    fs,
    io::{self, Seek},
    os::unix::fs::MetadataExt,
    path::Path,
};

use anyhow::Context;
use fatfs::{Dir, FormatVolumeOptions};
use gpt::{disk::LogicalBlockSize, mbr::ProtectiveMBR, partition_types};
use tempfile::NamedTempFile;

const KB: u64 = 1024;
const MB: u64 = KB * KB;
const CLUSTER_SIZE: u64 = 4 * KB;

/// Create a FAT filesystem image with the given files.
pub fn create_fat_fs(
    fat_image_file: &NamedTempFile,
    files: Vec<(&str, &str)>,
) -> anyhow::Result<()> {
    // Calculate the total size of all files.
    let mut total_size = 0;
    for (source_path, _) in &files {
        let size = fs::metadata(source_path)
            .with_context(|| format!("failed to get metadata from {}", source_path))?
            .len();
        total_size += size;
    }
    // Calculate Size of the data region (exlcuding root directory region).
    let data_region_size = total_size.div_ceil(CLUSTER_SIZE) * CLUSTER_SIZE;
    // Pad the total size to the nearest MB + an extra MB.
    let total_padded_size = data_region_size.div_ceil(MB) * MB + MB;
    fat_image_file
        .as_file()
        .set_len(total_padded_size)
        .with_context(|| format!("failed to set FAT image to size {}", total_padded_size))?;

    let options = FormatVolumeOptions::new()
        .volume_label(*b"__Meow_OS__")
        .bytes_per_cluster(CLUSTER_SIZE as u32);
    fatfs::format_volume(fat_image_file, options).with_context(|| {
        format!(
            "failed to format FAT image at {}",
            fat_image_file.path().display()
        )
    })?;

    let fs =
        fatfs::FileSystem::new(fat_image_file, fatfs::FsOptions::new()).with_context(|| {
            format!(
                "failed to create FAT file system at {}",
                fat_image_file.path().display()
            )
        })?;
    let root_dir = fs.root_dir();

    // Insert files into the FAT filesystem.
    for (source_path, target_path) in files {
        // Create all directories in the path except the last one (we assume last part of path is file).
        fatfs_create_dir_all_exclude_last(&root_dir, Path::new(target_path))
            .with_context(|| format!("failed to create parent directories for {}", target_path))?;

        // Create and copy file into FAT fs.
        let mut file = fs::File::open(source_path)?;
        let mut new_fat_file = root_dir.create_file(target_path)?;
        io::copy(&mut file, &mut new_fat_file)?;
    }

    Ok(())
}

/// Create all components excluding last component in the given path.
fn fatfs_create_dir_all_exclude_last(
    root_dir: &Dir<&NamedTempFile>,
    path: &Path,
) -> anyhow::Result<()> {
    let mut components = path.components().peekable();
    let mut curr_dir = root_dir.clone();

    while let Some(component) = components.next() {
        // Always skip last component.
        if components.peek().is_none() {
            break;
        }

        match component {
            std::path::Component::Normal(name) => {
                curr_dir.create_dir(&name.to_string_lossy())?;
                curr_dir = curr_dir.open_dir(&name.to_string_lossy())?; // "cd" into the new directory
            }
            _ => {
                return Err(anyhow::Error::msg(
                    "attempting to create invalid path in FAT image",
                ))
            }
        }
    }

    Ok(())
}

/// Create a GPT disk image with the given FAT filesystem image.
pub fn create_gpt_disk(disk_image_path: &Path, fat_image_path: &Path) -> anyhow::Result<()> {
    let mut disk_image = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .truncate(true)
        .open(disk_image_path)
        .with_context(|| {
            format!(
                "failed to open disk image at '{}'",
                disk_image_path.display()
            )
        })?;

    let fat_size = fs::metadata(&fat_image_path)
        .with_context(|| {
            format!(
                "failed to get fat image metadata from '{}'",
                fat_image_path.display()
            )
        })?
        .size();
    let disk_size = fat_size + (128 * u64::from(LogicalBlockSize::Lb512)); // Reserve space for 128 partition entries. Each is one 512 bytes.
    disk_image.set_len(disk_size).with_context(|| {
        format!(
            "failed to set disk image size at '{}' to {}",
            disk_image_path.display(),
            disk_size,
        )
    })?;

    // Not sure if PMBR is required for UEFI, but OVMF seems to require it.
    // Calculate size of disk in logical blocks (0-indexed).
    let lb_size: u64 = (disk_size / u64::from(LogicalBlockSize::Lb512)) - 1;
    let pmbr = ProtectiveMBR::with_lb_size(u32::try_from(lb_size).unwrap_or(0xFF_FF_FF_FF));
    pmbr.overwrite_lba0(&mut disk_image)
        .context("failed to write protective MBR to LBA0")?;

    let mut gpt = gpt::GptConfig::new()
        .writable(true)
        .initialized(false)
        .logical_block_size(LogicalBlockSize::Lb512)
        .open_from_device(Box::new(&disk_image))
        .with_context(|| format!("failed to open open GPT at '{}'", disk_image_path.display()))?;

    // Add EFI partition for the bootloader.
    gpt.update_partitions(Default::default())
        .context("failed to prep for GPT partition update")?;
    let partition_id = gpt
        .add_partition("boot", fat_size, partition_types::EFI, 0, None)
        .context("failed to insert EFI into GPT")?;
    let start_offset = gpt
        .partitions()
        .get(&partition_id)
        .unwrap()
        .bytes_start(LogicalBlockSize::Lb512)
        .context("failed to get start offset of EFI partition")?;

    gpt.write()
        .with_context(|| format!("failed to write GPT to '{}'", disk_image_path.display()))?;

    // Copy FAT filesystem image into disk image.
    disk_image
        .seek(io::SeekFrom::Start(start_offset))
        .with_context(|| {
            format!(
                "failed to seek to offset of {} at '{}'",
                start_offset,
                disk_image_path.display()
            )
        })?;
    io::copy(&mut fs::File::open(fat_image_path)?, &mut disk_image).with_context(|| {
        format!(
            "failed to perform IO copy from '{}' to '{}'",
            fat_image_path.display(),
            disk_image_path.display()
        )
    })?;

    Ok(())
}
