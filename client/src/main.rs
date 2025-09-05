use clap::Parser;
use crate::cli::ApplicationArguments;

mod network;
mod cli;

pub mod proto {
    pub mod comms {
        include!(concat!(env!("OUT_DIR"), "/comms.rs"));
    }
    pub mod constant {
        include!(concat!(env!("OUT_DIR"), "/constant.rs"));
    }
}

fn main() {
    let args = ApplicationArguments::parse();
    println!("{:?}", args.command);
    println!("hello world");
}