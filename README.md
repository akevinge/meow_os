# Meow OS

Heavily documented, learning OS dedicated to my favorite animal 🐈💕

## Usage

1. ```bash
   cargo run -p image_builder
   ```

2. ```bash
   qemu-system-x86_64 -drive format=raw,file=bin/disk_image.bin
   ```

## TODOS:

- [ ] Bootloader (in-progress)
- [ ] Long mode (?)
- [ ] HW Interrupts
- [ ] Paging
