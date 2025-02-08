pub enum Args {
    Interactive,
    FEN(String),
}
pub fn print_usage() {
    let exec = std::env::args().next().unwrap();
    println!("Usage: {exec} (--fen [FEN])?")
}

// TODO: we will eventually want an actual proper system for this.
fn unexpected_arg(invalid: String) {
    println!("Invalid argument {invalid}");
}
impl Args {
    pub fn parse() -> Option<Self> {
        let mut args = std::env::args();
        let _exec = args.next()?;
        if let Some(arg) = args.next() {
            match arg.as_str() {
                "--fen" => Some(Args::FEN(args.next()?)),
                _incorrect => {
                    unexpected_arg(arg);
                    None
                }
            }
        } else {
            Some(Args::Interactive)
        }
    }
}
