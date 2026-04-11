fn main() {
    if let Err(error) = stf_sir::cli::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
