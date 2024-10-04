use actix_web::{post, get, web, App, HttpResponse, HttpServer, Responder};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use clap::Parser;
use a2::{
    client::ClientConfig, Client, DefaultNotificationBuilder, Endpoint, NotificationBuilder, NotificationOptions,
};
use serde::{Deserialize, Serialize};
// use sled::Config;
mod db_operations;

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
struct SPawConfig {
    key_file_path: String,
    team_id: String,
    key_id: String,
    topic: String,
}

#[derive(Serialize)]
struct PushResponse {
    success: bool,
    message: String,
}

// 新增用于接收推送信息的结构体
#[derive(Deserialize)]
struct PushInfo {
    // device_token: String,
    user_token: Option<String>,
    message: String,
    sandbox: bool,
    category: Option<String>, // 新增 category 字段
}

fn load_config(cli: &Cli) -> SPawConfig {
    // 首先尝试从指定的配置文件或默认的 config.json 加载
    let mut config = cli.config.as_ref()
        .map(|path| read_config_file(path))
        .unwrap_or_else(|| read_config_file("config.json"))
        .unwrap_or_else(|_| SPawConfig {
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


fn read_config_file<P: AsRef<Path>>(path: P) -> Result<SPawConfig, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: SPawConfig = serde_json::from_str(&contents)?;
    Ok(config)
}


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "message": "Server is running"
    }))
}


#[post("/push/{user_token}")]
async fn send_push(user_token: web::Path<String>,push_info: web::Json<PushInfo>, config: web::Data<SPawConfig>) -> HttpResponse {

    // 从URL路径中获取user_token
    let user_token = user_token.into_inner();

    // 更新PushInfo结构体中的user_token
    let mut push_info = push_info.into_inner();
    push_info.user_token = Some(user_token);
    
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
    let mut builder = DefaultNotificationBuilder::new()
        .set_body(&push_info.message)
        .set_sound("default")
        .set_title("消息")
        .set_badge(1u32)
        .set_mutable_content(); // 添加这行来设置 mutable-content 标志

    // 检查是否需要设置 category
    if let Some(category) = &push_info.category {
        builder = builder.set_category(category);
    }else{
        builder = builder.set_category("QUICK_ACTIONS_CATEGORY")
    }
    
    // // 检查是否需要设置 category
    // if let Some(category) = push_info.category { // 假设 PushInfo 结构体中新增了 category 字段
    //     builder = builder.set_category(&category);
    // }

    let options = NotificationOptions {
        apns_topic: Some(&topic),
        ..Default::default()
    };

    // 从数据库获取设备令牌
    let device_token = match &push_info.user_token {
        Some(user_token) => match db_operations::get_device_token(web::Json(user_token.clone())).await {
            Some(token) => token,
            None => return HttpResponse::NotFound().json("未找到与用户令牌关联的设备"),
        },
        None => return HttpResponse::BadRequest().json("未提供用户令牌"),
    };

    // 检查设备令牌是否存在
    if device_token.is_empty() {
        return HttpResponse::NotFound().json("未找到与用户令牌关联的设备");
    }

    println!("获取到的设备令牌: {}", device_token);

    let mut payload = builder.build(&device_token, options);
    // 添加自定义数据
    if let Err(e) = payload.add_custom_data("text_to_copy", &push_info.message) {
        return HttpResponse::InternalServerError().json(PushResponse {
            success: false,
            message: format!("添加自定义数据失败: {}", e),
        });
    }

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
    // let db_path = Path::new("user-device.db");
    // let sled_config = Config::new().path(db_path);  // 使用 Config 设置路径
    // let sled_db = sled_config.open().expect("Failed to open/create database");

    let cli = Cli::parse();
    let config = load_config(&cli);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .service(db_operations::register_device)
            .service(send_push)
            .service(health_check)
            .service(hello)
            .service(db_operations::push_message) // 注册推送消息的路由
            .service(
                web::resource("/remove/{device_token}")
                    .route(web::delete().to(db_operations::remove_device)),
            )
    })
    .bind("0.0.0.0:8080")? // 监听所有IP地址的8080端口
    .run()
    .await
}