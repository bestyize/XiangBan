use server_xiaoban_market::marketplace::{self, Options};

#[tokio::main]
async fn main() {
    let opts = match Options::parse(std::env::args().skip(1)) {
        Ok(v) => v,
        Err(e) => { eprintln!("{e}"); std::process::exit(2); }
    };
    match marketplace::run(&opts, |line| println!("{line}")).await {
        Ok(s) if s.failed == 0 => {},
        Ok(_) => std::process::exit(1),
        Err(e) => { eprintln!("{}", opts.redact(&e)); std::process::exit(1); }
    }
}
