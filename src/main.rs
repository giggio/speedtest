#[macro_use]
mod macros;
mod alert;
mod args;
mod run;
use args::{Args, Command};

static mut VERBOSE: bool = false;

fn main() {
    match run() {
        Err(None) => std::process::exit(1),
        Err(Some(x)) => {
            eprintln!("{}", x);
            std::process::exit(1);
        }
        Ok(_) => std::process::exit(0),
    }
}

fn run() -> Result<(), Option<String>> {
    let args = Args::new();
    unsafe {
        VERBOSE = args.verbose;
    }
    printlnv!("Args are {:?}.", args);
    match args.command {
        Some(config) => match config {
            Command::Run(run) => run::run(run),
            Command::Alert(alert) => alert::alert(alert),
        },
        _ => Err(None),
    }
}
