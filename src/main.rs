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
    /// 快速入门指南
    Intro,
    /// 初始化设备配置
    Init {
        /// 手机 IP 地址
        #[arg(short, long)]
        ip: Option<String>,
        /// SSH 端口
        #[arg(short, long)]
        port: Option<u16>,
        /// SSH 用户名
        #[arg(short = 'u', long)]
        user: Option<String>,
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 查看 SSH 和 Tailscale 连接状态
    Status {
        #[arg(long)]
        json: bool,
    },
    /// 启动 SSH Server
    Start,
    /// 停止 SSH Server
    Stop,
    /// 发送文件到手机
    Send {
        /// 要发送的文件或文件夹路径
        file: String,
        /// 递归发送文件夹
        #[arg(short, long)]
        recursive: bool,
        /// 强制使用 rsync
        #[arg(long)]
        rsync: bool,
        /// 强制使用 scp
        #[arg(long)]
        scp: bool,
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 从手机拉取文件
    Pull {
        /// 要拉取的文件名
        file: Option<String>,
        /// 列出远程文件
        #[arg(short, long)]
        list: bool,
        /// 拉取全部文件
        #[arg(short, long)]
        all: bool,
        /// 保存目录
        #[arg(short, long)]
        output: Option<String>,
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 接收手机发送的文件
    Receive {
        /// 实时监控新文件
        #[arg(long)]
        watch: bool,
    },
    /// 列出已接收的文件
    List {
        /// 显示隐藏文件
        #[arg(short, long)]
        all: bool,
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 查看传输历史记录
    History {
        /// 显示全部记录
        #[arg(long)]
        all: bool,
        /// 清除所有历史
        #[arg(long)]
        clear: bool,
        /// 只保留最近 n 条记录
        #[arg(long)]
        keep: Option<usize>,
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 配置管理
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// 启动 HTTP 文件服务
    Serve {
        /// 指定端口
        #[arg(short, long)]
        port: Option<u16>,
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 发现局域网设备
    Discover {
        /// 搜索超时（秒）
        #[arg(short, long, default_value = "5")]
        timeout: u64,
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
}

#[derive(clap::Subcommand)]
enum ConfigAction {
    /// 显示当前配置
    Show,
    /// 设置手机 IP 地址
    SetIp { ip: String },
    /// 获取当前手机 IP
    GetIp,
    /// 设置手机 SSH 用户名
    SetUser { user: String },
    /// 设置设备别名
    SetAlias { ip: String, name: String },
    /// 自动检测设备
    AutoDetect,
    /// 生成 SSH 密钥对
    Keygen {
        /// 强制覆盖现有密钥
        #[arg(short, long)]
        force: bool,
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
    /// 部署公钥到手机
    KeyCopy {
        /// JSON 格式输出
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    ctrlc::set_handler(|| {
        println!("\n👋 已停止");
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Intro) => commands::intro::handle_intro()?,
        Some(Commands::Init { ip, port, user, json }) => commands::init::handle_init(ip, port, user, json)?,
        Some(Commands::Config { action }) => match action {
            ConfigAction::Show => commands::config::handle_show()?,
            ConfigAction::SetIp { ip } => commands::config::handle_set_ip(&ip)?,
            ConfigAction::GetIp => commands::config::handle_get_ip()?,
            ConfigAction::SetUser { user } => commands::config::handle_set_user(&user)?,
            ConfigAction::SetAlias { ip, name } => commands::config::handle_set_alias(&ip, &name)?,
            ConfigAction::AutoDetect => println!("Auto-detect 未实现"),
            ConfigAction::Keygen { force, json } => commands::config::handle_keygen(force, json)?,
            ConfigAction::KeyCopy { json } => commands::config::handle_key_copy(json)?,
        },
        Some(Commands::Status { json }) => commands::status::handle_status(json)?,
        Some(Commands::Start) => commands::start::handle_start()?,
        Some(Commands::Stop) => commands::stop::handle_stop()?,
        Some(Commands::Send { file, recursive, rsync, scp, json }) => {
            commands::send::handle_send(&file, recursive, rsync, scp, json)?;
        }
        Some(Commands::Pull { file, list, all, output, json }) => {
            commands::pull::handle_pull(file, list, all, output, json)?;
        }
        Some(Commands::Receive { watch }) => commands::receive::handle_receive(watch)?,
        Some(Commands::List { all, json }) => commands::list::handle_list(all, json)?,
        Some(Commands::History { all, clear, keep, json }) => commands::history::handle_history(all, clear, keep, json)?,
        Some(Commands::Serve { port, json }) => {
            commands::serve::handle_serve(port, json)?;
        }
        Some(Commands::Discover { timeout, json }) => {
            commands::discover::handle_discover(timeout, json)?;
        }
        None => println!("使用 --help 查看帮助"),
    }
    Ok(())
}
