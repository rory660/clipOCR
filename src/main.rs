use clap::Parser;
use clipocr::cli::Args;

fn main() {
    let args = Args::parse();
    match clipocr::run(args) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("clipocr: error: {err}");
            std::process::exit(err.exit_code());
        }
    }
}
