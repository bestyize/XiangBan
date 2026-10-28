//! 联调工具：不启动桌面应用，直接对着论坛检查一个内容包能不能被客户端库下载、校验和读懂。
//!   cargo run --example fetch -- https://chat.xiangban.xyz 12 [version]
use xiaoban_community::{inspect, Community};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (origin, id) = (args.next().ok_or("用法：fetch <论坛地址> <内容包编号> [版本]")?, args.next().ok_or("缺少内容包编号")?);
    let version = args.next().map(|v| v.parse::<i32>().map_err(|_| "版本需要是数字")).transpose()?;
    let client = Community::new(&origin)?;
    let info = client.download_info(id.parse().map_err(|_| "编号需要是数字")?, version, None)?;
    println!("{} v{}（{} 字节，信任等级 {}，一键应用：{}）", info.kind, info.label, info.size, info.trust_level, info.one_click);
    let bytes = client.fetch(&info)?;
    println!("下载并通过 SHA-256 校验：{}", info.sha256);
    let found = inspect(&bytes, &info.kind)?;
    println!("manifest：{}", found.manifest);
    println!("文件：{}", found.files.join(", "));
    if let Some(payload) = found.payload {
        println!("payload：{payload}");
    }
    Ok(())
}
