use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
        local_path: String,
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
    List {},

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