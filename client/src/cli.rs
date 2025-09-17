use std::error::Error;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use redb::{Database, ReadableDatabase, ReadableTable};
use crate::cli;
use crate::tables::OBJECTS_LOCAL_TABLE;

#[derive(Debug, Subcommand)]
#[command(
    version = "",
    infer_subcommands = true,
    disable_version_flag = true,
    propagate_version = false,
    disable_help_subcommand = true,
    arg_required_else_help = true
)]
pub enum Commands {
    /// [REMOTE] Add a file or directory to VCS
    Add {
        /// Path to File or directory ["D:/Programming", ".", or "D:/Programming/config.txt"]
        path: PathBuf,
    },

    /// [REMOTE] Removes a file or directory to VCS
    Remove {
        /// Identifier for object either ID, or Path ["F2FF2697", "D:/Programming/File.txt"]
        target: String,
    },

    /// [LOCAL]  Syncs a file/directory to disk
    Sync {
        /// Identifier for object either ID, or Path ["F2FF2697", "D:/Programming/File.txt"]
        target: String,

        /// The destination directory or file path on the local machine
        local_path: PathBuf,
    },

    /// [LOCAL]  Stops a file/directory from being synced.
    Dsync {
        /// Identifier for object either ID, or Path ["F2FF2697", "D:/Programming/File.txt"]
        target: String,
    },

    /// List all synced files and directories
    List,
}

#[derive(Debug, Parser)]
#[command(
    name = "Dsync - EthoIRL",
    version = "",
    infer_subcommands = true,
    disable_version_flag = true,
    propagate_version = false,
    disable_help_subcommand = true,
    arg_required_else_help = false
)]
pub struct ApplicationArguments {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

pub fn hex_id_to_u8_array(hex_string: &String) -> Result<[u8; 4], Box<dyn Error>> {
    if hex_string.len() != 8 {
        return Err("Hex string not the valid 8 character length".into())
    }

    let bytes: Result<Vec<u8>, _> = (0..4)
        .map(|i| u8::from_str_radix(&hex_string[i*2..i*2+2], 16))
        .collect();

    match bytes?.try_into() {
        Ok(vec) => Ok(vec),
        Err(_) => Err("Invalid hex digit found".into()),
    }
}

// Target can be either a 4 byte hex string or a path
pub fn get_id_from_target(database: &Database, target: &String) -> Result<[u8; 4], Box<dyn Error>> {
    let id = match cli::hex_id_to_u8_array(&target) {
        Ok(id) => id,
        Err(_) => {
            // Assume hex isn't an ID
            let potential_path = PathBuf::from(&target);

            if !potential_path.exists() {
                return Err("Path or ID is correct".into());
            }

            let read_txn = database.begin_read()?;
            let object_table = read_txn.open_table(OBJECTS_LOCAL_TABLE)?;

            let potential_object = object_table.iter()?.find(|result| {
                if let Ok((_, value)) = result {
                    if value.value().eq_ignore_ascii_case(&target) {
                        return true;
                    }
                }

                false
            });

            let object = match potential_object {
                Some(object) => object,
                None => return Err("Couldn't find a path".into())
            }?;

            object.0.value()
        }
    };

    Ok(id)
}