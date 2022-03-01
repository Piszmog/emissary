use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, default_value_t = String::from("./config.toml"))]
    pub config_file: String,
}

pub fn parse_args() -> Args {
    Args::parse()
}