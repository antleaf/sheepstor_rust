use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "sheepstor", version)]
pub struct Cli {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    /// Config file path
    #[clap(global = true)]
    #[clap(long, short = 'c', default_value_t = String::from("./config.yaml"))]
    pub config: String,

    ///Enable debug logging
    #[clap(global = true)]
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Help message for Server.
    Server {
        /// Port number
        #[clap(long, short = 'p',default_value_t = 3000)]
        port: u16,
    },
    /// Help message for Update.
    Update {
        /// Site(s) to update
        #[clap(long, short = 's')]
        sites: String,
    },
    /// Help message for Push.
    Push {
        /// Site(s) to update
        #[clap(long, short = 's')]
        site: String,
    },
}

