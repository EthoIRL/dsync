use crate::proto::constant::ChunkSize;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::SystemTime;
use xxhash_rust::xxh3::xxh3_64;

pub fn hash_object(object_path: &PathBuf) -> Result<u64, Box<dyn Error>> {
    let object_path_string = object_path.to_string_lossy();

    if object_path.is_dir() {
        Ok(xxh3_64(object_path_string.as_bytes()))
    }
    else
    {
        let mut file = File::open(object_path.clone()).expect("Failed to open file... during traversal");

        let mut data: Vec<u8> = Vec::new();
        match file.read_to_end(&mut data) {
            Err(_) => Ok(xxh3_64(object_path_string.as_bytes())),
            Ok(size) => {
                if size == 0 {
                    Ok(xxh3_64(object_path_string.as_bytes()))
                } else {
                    Ok(xxh3_64(&data))
                }
            }
        }
    }
}

pub fn object_last_modified(object_path: &PathBuf) -> Result<u64, Box<dyn Error>> {
    if !object_path.exists() {
        return Err(format!("Object {:?} does not exist!", object_path).into())
    }

    let system_time = fs::metadata(object_path)?.modified()?;
    let timestamp = system_time.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();

    Ok(timestamp)
}

pub fn hash_file_chunks(object_path: &PathBuf) -> Result<Vec<u64>, Box<dyn Error>> {
    if object_path.is_dir() {
        return Err("Cannot hash directory into chunks".into());
    }

    let mut file = File::open(object_path.clone()).expect("Failed to open file... during traversal");
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;

    let mut chunk_hashes: Vec<u64> = Vec::new();
    for chunk in data.chunks(ChunkSize::Size as usize) {
        chunk_hashes.push(xxh3_64(chunk));
    };

    Ok(chunk_hashes)
}
