use anyhow::Result;
use clap::Parser;
use domains::show_domains;
use key::KeyCommands;
use query::QueryArgs;
use send::SendArgs;

mod abi;
mod domains;
mod key;
mod query;
mod send;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: SubCommand,
}

#[derive(Debug, Parser)]
enum SubCommand {
    /// Key commands, generate key, etc...
    #[command(subcommand)]
    Key(KeyCommands),
    Send(SendArgs),
    Query(QueryArgs),
    ShowDomains,
}

impl SubCommand {
    async fn process(self) -> Result<()> {
        match self {
            Self::Key(key_commands) => key_commands.process(),
            Self::ShowDomains => {
                show_domains();
                Ok(())
            }
            Self::Send(args) => args.process().await,
            Self::Query(args) => args.process().await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    cli.cmd.process().await
}
