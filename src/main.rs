use actix_web::{post, get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct PushMessage {
    token: String,
    message: String,
}
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/pushmessage")]
async fn push_message(msg: web::Json<PushMessage>) -> impl Responder {
    // 这里处理推送逻辑，比如调用APNs推送
    println!("Received token: {} with message: {}", msg.token, msg.message);
    
    // 返回成功响应
    HttpResponse::Ok().body("Message received and sent to APNs")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(push_message) // 注册推送消息的路由
    })
    .bind("0.0.0.0:8080")? // 监听所有IP地址的8080端口
    .run()
    .await
}