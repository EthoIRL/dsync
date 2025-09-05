use std::io::Result;
use std::path::PathBuf;
use prost_build::compile_protos;

const PROTO_DIR: &str = "../proto";

fn main() -> Result<()> {
    let mut proto_files: Vec<PathBuf> = Vec::new();
    traverse_proto_dir(PathBuf::from(&PROTO_DIR), &mut proto_files);

    println!("{:#?}", proto_files);

    compile_protos(&proto_files, &[PROTO_DIR])?;

    Ok(())
}

fn traverse_proto_dir(directory: PathBuf, protos: &mut Vec<PathBuf>) {
    std::fs::read_dir(&directory).unwrap()
        .for_each(|entry| {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    traverse_proto_dir(entry.path(), protos);
                }

                if entry.path().extension().is_some_and(|extension| extension == "proto") {
                    protos.push(entry.path());
                }
            }
        });
}