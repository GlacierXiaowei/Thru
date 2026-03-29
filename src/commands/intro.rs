use anyhow::Result;

pub fn handle_intro() -> Result<()> {
    println!(r#"🚀 Thru 快速入门

1. 配置手机信息
   thru init --ip <手机IP> --user <用户名>

2. 生成 SSH 密钥（免密登录）
   thru config keygen
   thru config key-copy   # 按提示在手机执行命令

3. 发送文件到手机
   thru send 文件路径

4. 从手机拉取文件
   thru pull --list       # 查看手机上的文件
   thru pull 文件名       # 拉取文件

💡 提示: 安装 rsync 可获得更快的大文件传输
   Windows: scoop install rsync
   手机: pkg install rsync

详细帮助: thru <命令> --help"#);
    Ok(())
}