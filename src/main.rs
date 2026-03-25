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
    Status {
        #[arg(long)]
        json: bool,
    },
    Start,
    Stop,
    Send {
        file: String,
        #[arg(short, long)]
        recursive: bool,
    },
    Receive {
        #[arg(long)]
        watch: bool,
    },
    List {
        #[arg(short, long)]
        all: bool,
    },
    History {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        clear: bool,
        #[arg(long)]
        keep: Option<usize>,
    },
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
        Some(Commands::Status { json }) => commands::status::handle_status(json)?,
        Some(Commands::Start) => commands::start::handle_start()?,
        Some(Commands::Stop) => commands::stop::handle_stop()?,
        Some(Commands::Send { file, recursive }) => commands::send::handle_send(&file, recursive)?,
        Some(Commands::Receive { watch }) => commands::receive::handle_receive(watch)?,
        Some(Commands::List { all }) => commands::list::handle_list(all)?,
        Some(Commands::History { all, clear, keep }) => commands::history::handle_history(all, clear, keep)?,
        None => println!("使用 --help 查看帮助"),
    }
    Ok(())
}
