use std::{env, process::exit};

use tokio::task::LocalSet;

use crate::args::ArgumentsRequest;

mod args;
mod server;
mod utils;

fn main() {
    let arguments = match args::parse_arguments(env::args()) {
        Err(err) => {
            eprintln!("{err}\n\nType 'whereyoufrom --help' for a help menu");
            exit(1);
        }
        Ok(arguments) => arguments,
    };

    let startup_args = match arguments {
        ArgumentsRequest::Version => {
            println!("{}", args::get_version_string());
            println!("GPS? Don't need that anymore ⌐■_■");
            return;
        }
        ArgumentsRequest::Help => {
            println!("{}", args::get_help_string());
            return;
        }
        ArgumentsRequest::Run(startup_args) => startup_args,
    };

    let maybe_runtime = tokio::runtime::Builder::new_current_thread().enable_all().build();
    match maybe_runtime {
        Ok(runtime) => LocalSet::new().block_on(&runtime, server::run_server(startup_args)),
        Err(err) => eprintln!("Failed to start Tokio runtime: {err}"),
    }
}
