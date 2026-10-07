use std::path::PathBuf;

/// 默认监听端口
pub const DEFAULT_PORT: u16 = 55005;
/// 默认监听地址
pub const DEFAULT_HOST: &str = "0.0.0.0";

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub data_dir: PathBuf,
    pub storage_dir: PathBuf,
    pub db_path: PathBuf,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
}

/// 命令行参数解析结果
#[derive(Debug, Default)]
struct CliArgs {
    host: Option<String>,
    port: Option<u16>,
    data_dir: Option<PathBuf>,
}

impl CliArgs {
    /// 解析命令行参数，支持：
    ///   -p, --port <PORT>        指定监听端口
    ///   -H, --host <HOST>        指定监听地址
    ///   -d, --data-dir <DIR>     指定数据目录
    ///   -h, --help               打印帮助
    fn parse() -> Self {
        let mut args = CliArgs::default();
        let mut iter = std::env::args().skip(1);

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "-p" | "--port" => {
                    let value = iter.next().unwrap_or_else(|| {
                        eprintln!("错误：选项 {} 缺少端口号参数", arg);
                        std::process::exit(2);
                    });
                    args.port = Some(parse_port(&value));
                }
                "-H" | "--host" => {
                    args.host = iter.next();
                }
                "-d" | "--data-dir" => {
                    args.data_dir = iter.next().map(PathBuf::from);
                }
                // 支持 --port=55005 这种等号写法
                _ if arg.starts_with("--port=") => {
                    args.port = Some(parse_port(&arg["--port=".len()..]));
                }
                _ if arg.starts_with("--host=") => {
                    args.host = Some(arg["--host=".len()..].to_string());
                }
                _ if arg.starts_with("--data-dir=") => {
                    args.data_dir = Some(PathBuf::from(&arg["--data-dir=".len()..]));
                }
                _ => {
                    eprintln!("错误：无法识别的参数 '{}'（使用 --help 查看用法）", arg);
                    std::process::exit(2);
                }
            }
        }

        args
    }
}

fn parse_port(value: &str) -> u16 {
    value.parse::<u16>().unwrap_or_else(|_| {
        eprintln!("错误：端口号 '{}' 无效，应为 1-65535 之间的整数", value);
        std::process::exit(2);
    })
}

fn print_help() {
    println!(
        r#"PanDown 极简网络云盘

用法:
    pandown-backend [选项]

选项:
    -p, --port <PORT>        指定监听端口 (默认: {port})
    -H, --host <HOST>        指定监听地址 (默认: {host})
    -d, --data-dir <DIR>     指定数据目录 (默认: pandown_data)
    -h, --help               打印本帮助信息

示例:
    pandown-backend                     # 使用默认端口 {port}
    pandown-backend --port 9000         # 监听 9000 端口
    pandown-backend -p 9000             # 简写形式
    pandown-backend --port=9000         # 等号形式

环境变量 (优先级低于命令行参数):
    PORT, HOST, DATA_DIR, JWT_SECRET
"#,
        port = DEFAULT_PORT,
        host = DEFAULT_HOST,
    );
}

impl AppConfig {
    pub fn load() -> Self {
        let cli = CliArgs::parse();

        // 优先级：命令行参数 > 环境变量 > 默认值
        let host = cli
            .host
            .or_else(|| std::env::var("HOST").ok())
            .unwrap_or_else(|| DEFAULT_HOST.to_string());

        let port = cli
            .port
            .or_else(|| std::env::var("PORT").ok().and_then(|p| p.parse().ok()))
            .unwrap_or(DEFAULT_PORT);

        let data_dir = cli
            .data_dir
            .or_else(|| std::env::var("DATA_DIR").ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("pandown_data"));

        let storage_dir = data_dir.join("storage");
        let db_path = data_dir.join("pandown.db");

        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "pandown-super-secret-key-2026-apple-ui".to_string());

        Self {
            host,
            port,
            data_dir,
            storage_dir,
            db_path,
            jwt_secret,
            jwt_expiration_hours: 24 * 7, // 7 天有效期
        }
    }
}
