use clap::Parser;
use anyhow::Result;

mod core;
mod commands;
mod utils;

#[derive(Parser)]
#[command(name = "thru", version, about = "手机 - 电脑文件互传工具")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Status,
    Start,
    Stop,
    Send { file: String },
    Receive,
    List,
    History,
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(clap::Subcommand)]
enum ConfigAction {
    Show,
    SetIp { ip: String },
    GetIp,
    SetAlias { ip: String, name: String },
    AutoDetect,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Config { action }) => match action {
            ConfigAction::Show => commands::config::handle_show()?,
            ConfigAction::SetIp { ip } => commands::config::handle_set_ip(&ip)?,
            ConfigAction::GetIp => commands::config::handle_get_ip()?,
            ConfigAction::SetAlias { ip, name } => commands::config::handle_set_alias(&ip, &name)?,
            ConfigAction::AutoDetect => println!("Auto-detect 未实现"),
        },
        Some(Commands::Status) => println!("Status command"),
        Some(Commands::Start) => println!("Start command"),
        Some(Commands::Stop) => println!("Stop command"),
        Some(Commands::Send { file }) => println!("Send: {}", file),
        Some(Commands::Receive) => println!("Receive command"),
        Some(Commands::List) => println!("List command"),
        Some(Commands::History) => println!("History command"),
        None => println!("使用 --help 查看帮助"),
    }
    Ok(())
}
