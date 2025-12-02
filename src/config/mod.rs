use clap::Parser;

#[derive(Debug, Parser, Clone)]
pub struct Config {
    #[clap(long, env)]
    pub api_key: String,
    #[clap(long, env, default_value = "uploads")]
    pub upload_dir: String,
}
