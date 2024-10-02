use actix_web::{post, get, web, App, HttpResponse, HttpServer, Responder};
use std::fs::File;
use a2::{
    client::ClientConfig, Client, DefaultNotificationBuilder, Endpoint, NotificationBuilder, NotificationOptions,
};
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use std::env;


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
async fn send_push(push_info: web::Json<PushInfo>) -> HttpResponse {
    // 读取环境变量
    let key_file = env::var("KEY_FILE_PATH").expect("KEY_FILE_PATH not set");
    let team_id = env::var("TEAM_ID").expect("TEAM_ID not set");
    let key_id = env::var("KEY_ID").expect("KEY_ID not set");
    let topic =  env::var("TOPIC").ok();

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
        apns_topic: topic.as_deref(),
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
    dotenv().ok(); // 确保环境变量已加载

    HttpServer::new(|| {
        App::new()
            .service(send_push)
            .service(health_check)
            .service(hello)
            .service(push_message) // 注册推送消息的路由
    })
    .bind("0.0.0.0:8080")? // 监听所有IP地址的8080端口
    .run()
    .await
}