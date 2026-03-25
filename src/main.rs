use clap::Parser;

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
    Config,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Status) => println!("Status command"),
        Some(Commands::Start) => println!("Start command"),
        Some(Commands::Stop) => println!("Stop command"),
        Some(Commands::Send { file }) => println!("Send: {}", file),
        Some(Commands::Receive) => println!("Receive command"),
        Some(Commands::List) => println!("List command"),
        Some(Commands::History) => println!("History command"),
        Some(Commands::Config) => println!("Config command"),
        None => println!("使用 --help 查看帮助"),
    }
}
