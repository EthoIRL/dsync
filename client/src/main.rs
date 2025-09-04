use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum ClientCommands {
    // TODO: Maybe we want to let the user determine if they want to to force save? e.g. save the current iteration to remote vcs or pull down from vcs after syncing backup?
    /// Resyncs a file or directory
    /// - Saves the current iteration to the VCS
    /// - Re-enables syncing if previously disabled [NOTE: Can be used as "``git push``"]
    #[structopt(about = "Resyncs a file or directory\n- Saves the current iteration to the VCS\n- Re-enables syncing if previously disabled [NOTE: Can be used as \"git push\"]")]
    Resync {
        /// Either an VCS ID, or a Path [2, "D:/Programming/File.txt"]
        target: String,
    },

    /// Removes a file or directory from being synced, without changing global syncing.
    Dsync {
        /// Either an VCS ID, or a Path [2, "D:/Programming/File.txt"]
        target: String,
    }
}

#[derive(Debug, StructOpt)]
pub enum ServerCommands {
    /// Add a file or directory to the synced list
    Add {
        /// File of directory ["D:/Programming", or ".", or "D:/Programming/config.txt"]
        path: PathBuf
    },

    /// Remove's a file or directory from the synced list
    Remove {
        /// Either an VCS ID, or a Path [2, "D:/Programming/File.txt"]
        target: String
    },

    /// Sync's a file/directory to disk at a specified location
    Sync {
        /// Either an VCS ID, or a Path [2, "D:/Programming/File.txt"]
        target: String,
        /// The target local for this file or directory on the local machine
        local_path: String
    }
}

#[derive(Debug, StructOpt)]
pub enum Commands {
    /// List all synced files and directories
    List {},

    /// Remote source control
    Remote(ServerCommands),
    /// Local control commands
    Local(ClientCommands),

    /// Forces a global sync
    Sync,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "dsync")]
pub struct ApplicationArguments {
    #[structopt(subcommand)]
    pub command: Option<Commands>,
}


fn main() {
    let a = ApplicationArguments::from_args();
    println!("{:?}", a.command);
    println!("hello world");
}