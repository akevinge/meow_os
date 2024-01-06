use std::env;

mod build;
mod run;

const HELP: &str = "Usage: image_builder [help|build|run|all]
    help: Display this message.
    build: Build the bootloader and kernel.
    run: Run the disk image.
    all: Build the bootloader and kernel, then run the disk image.";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("{}", HELP);
        return;
    }

    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| "bin".to_string());
    let image_path = out_dir.to_owned() + "/disk_image.bin";

    match args[1].as_str() {
        "help" => println!("{}", HELP),
        "build" => build::run(&out_dir, &image_path),
        "run" => run::run(&image_path),
        "all" => {
            build::run(&out_dir, &image_path);
            run::run(&image_path);
        }
        _ => println!("{}", HELP),
    }
}
