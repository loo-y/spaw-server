use actix_web::{post, get, web, App, HttpResponse, HttpServer, Responder};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use clap::Parser;
use a2::{
    client::ClientConfig, Client, DefaultNotificationBuilder, Endpoint, NotificationBuilder, NotificationOptions,
};
use serde::{Deserialize, Serialize};
// use dotenv::dotenv;
// use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 可选的配置文件路径
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// 密钥文件路径
    #[arg(short, long)]
    key_file_path: Option<String>,

    /// 团队 ID
    #[arg(short, long)]
    team_id: Option<String>,

    /// 密钥 ID
    #[arg(short, long)]
    key_id: Option<String>,

    /// 主题（通常是应用的 Bundle ID）
    #[arg(short, long)]
    topic: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    key_file_path: String,
    team_id: String,
    key_id: String,
    topic: String,
}


#[derive(Deserialize)]
struct PushMessage {
    device_token: String,
    body: String,
}

#[derive(Serialize)]
struct PushResponse {
    success: bool,
    message: String,
}

// 新增用于接收推送信息的结构体
#[derive(Deserialize)]
struct PushInfo {
    device_token: String,
    message: String,
    sandbox: bool
}

fn load_config(cli: &Cli) -> Config {
    // 首先尝试从指定的配置文件或默认的 config.json 加载
    let mut config = cli.config.as_ref()
        .map(|path| read_config_file(path))
        .unwrap_or_else(|| read_config_file("config.json"))
        .unwrap_or_else(|_| Config {
            key_file_path: String::new(),
            team_id: String::new(),
            key_id: String::new(),
            topic: String::new(),
        });

    // 然后用命令行参数覆盖配置文件中的值
    if let Some(key_file_path) = &cli.key_file_path {
        config.key_file_path = key_file_path.clone();
    }
    if let Some(team_id) = &cli.team_id {
        config.team_id = team_id.clone();
    }
    if let Some(key_id) = &cli.key_id {
        config.key_id = key_id.clone();
    }
    if let Some(topic) = &cli.topic {
        config.topic = topic.clone();
    }

    config
}


fn read_config_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/pushmessage")]
async fn push_message(msg: web::Json<PushMessage>) -> impl Responder {
    // 这里处理推送逻辑，比如调用APNs推送
    println!("Received token: {} with message: {}", msg.device_token, msg.body);
    
    // 返回成功响应
    HttpResponse::Ok().body("Message received and sent to APNs")
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "message": "Server is running"
    }))
}


#[post("/send_push")]
async fn send_push(push_info: web::Json<PushInfo>, config: web::Data<Config>) -> HttpResponse {

    // 读取环境变量
    // let key_file = env::var("KEY_FILE_PATH").expect("KEY_FILE_PATH not set");
    // let team_id = env::var("TEAM_ID").expect("TEAM_ID not set");
    // let key_id = env::var("KEY_ID").expect("KEY_ID not set");
    // let topic =  env::var("TOPIC").ok();

    println!("使用的密钥文件路径: {}", config.key_file_path);
    println!("使用的团队 ID: {}", config.team_id);
    println!("使用的密钥 ID: {}", config.key_id);
    println!("使用的主题: {}", config.topic);

    let key_file = config.key_file_path.clone();
    let team_id = config.team_id.clone();
    let key_id = config.key_id.clone();
    let topic  = config.topic.clone();


    // 读取私钥文件
    let mut private_key = match File::open(&key_file) {
        Ok(file) => file,
        Err(e) => return HttpResponse::InternalServerError().json(format!("无法打开私钥文件: {}", e)),
    };

    // 构建客户端配置
    let endpoint = if push_info.sandbox {
        Endpoint::Sandbox
    } else {
        Endpoint::Production
    };
    let client_config = ClientConfig::new(endpoint);

    // 创建APNs客户端
    let client = match Client::token(&mut private_key, &key_id, &team_id, client_config) {
        Ok(client) => client,
        Err(e) => return HttpResponse::InternalServerError().json(format!("创建APNs客户端失败: {}", e)),
    };

    // 构建通知
    let builder = DefaultNotificationBuilder::new()
        .set_body(&push_info.message)
        .set_sound("default")
        .set_badge(1u32);

    let options = NotificationOptions {
        apns_topic: Some(&topic),
        ..Default::default()
    };

    let payload = builder.build(&push_info.device_token, options);

    // 发送通知
    match client.send(payload).await {
        Ok(response) => HttpResponse::Ok().json(PushResponse {
            success: true,
            message: format!("通知发送成功: {:?}", response),
        }),
        Err(e) => HttpResponse::InternalServerError().json(PushResponse {
            success: false,
            message: format!("发送通知失败: {}", e),
        }),
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // dotenv().ok(); // 确保环境变量已加载

    let cli = Cli::parse();
    let config = load_config(&cli);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .service(send_push)
            .service(health_check)
            .service(hello)
            .service(push_message) // 注册推送消息的路由
    })
    .bind("0.0.0.0:8080")? // 监听所有IP地址的8080端口
    .run()
    .await
}