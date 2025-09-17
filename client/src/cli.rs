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
pub enum ClientCommands {
    /// Resyncs a file or directory
    Resync {
        /// Either a VCS ID, or a Path [2, "D:/Programming/File.txt"]
        target: String,
    },

    /// Removes a file or directory from being synced, without changing global syncing.
    Dsync {
        /// Either a VCS ID, or a Path [2, "D:/Programming/File.txt"]
        target: String,
    },
}

#[derive(Debug, Subcommand)]
#[command(
    version = "",
    infer_subcommands = true,
    disable_version_flag = true,
    propagate_version = false,
    disable_help_subcommand = true,
    arg_required_else_help = true,
)]
pub enum ServerCommands {
    /// Add a file or directory to the synced list
    Add {
        /// File or directory ["D:/Programming", ".", or "D:/Programming/config.txt"]
        path: PathBuf,
    },

    /// Removes a file or directory from the synced list
    Remove {
        /// Either a VCS ID, or a Path [2, "D:/Programming/File.txt"]
        target: String,
    },

    /// Syncs a file/directory to disk at a specified location
    Sync {
        /// Either a VCS ID, or a Path [2, "D:/Programming/File.txt"]
        target: String,
        /// The target location for this file or directory on the local machine
        local_path: PathBuf,
    },
}

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
    /// Remote source control commands
    Remote {
        #[command(subcommand)]
        command: ServerCommands
    },

    /// Local source control commands
    Local {
        #[command(subcommand)]
        command: ClientCommands
    },

    /// List all synced files and directories
    List,

    /// Forces a global sync
    Sync,
}

#[derive(Debug, Parser)]
#[command(
    name = "Dsync - EthoIRL",
    version = "",
    infer_subcommands = true,
    disable_version_flag = true,
    propagate_version = false,
    disable_help_subcommand = true,
    arg_required_else_help = true
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