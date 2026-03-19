use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Just print the openApi-spec and exit
    #[arg(short, long, default_value_t = false)]
    print_doc: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if args.print_doc {
        let doc = via_alias::api_doc::get_api_doc();
        match doc {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("Error during openApi-spec generation: {e}");
                std::process::exit(2)
            }
        }
    } else {
        if let Err(e) = via_alias::run_app().await {
            eprintln!("Via-Alias encountered an error: {e}");
            std::process::exit(1)
        }
    }
}
