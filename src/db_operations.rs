use actix_web::{web, HttpResponse, Responder, post};
use lazy_static::lazy_static;
use sled::Config;
use std::path::Path;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref DB: sled::Db = {
        let db_path = Path::new("user-device");
        let config = Config::new().path(db_path);
        config.open().expect("Failed to open/create database")
    };
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceInfo {
    device_token: String,
    user_token: String,
}

#[post("/save_token")]
pub async fn register_device(
    info: web::Json<DeviceInfo>,
) -> impl Responder {
    let device_token = info.device_token.clone();
    let user_token = info.user_token.clone();

    println!("device_token: {}", device_token);
    println!("user_token: {}", user_token);

    let result = DB.insert(user_token.as_bytes(), device_token.as_bytes());

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": true,
            "message": "Device registered/updated successfully"
        })),
        Err(e) => HttpResponse::InternalServerError().json(format!("Database error: {}", e)),
    }
}

#[derive(Deserialize)]
pub struct PushMessage {
    device_token: String,
    body: String,
}

#[post("/pushmessage")]
pub async fn push_message(msg: web::Json<PushMessage>) -> impl Responder {
    // 这里处理推送逻辑，比如调用APNs推送
    println!("Received token: {} with message: {}", msg.device_token, msg.body);
    
    // 返回成功响应
    HttpResponse::Ok().body("Message received and sent to APNs")
}

pub async fn remove_device(
    path: web::Path<String>,
) -> impl Responder {
    let device_token = path.into_inner();

    let result = DB.remove(device_token.as_bytes());

    match result {
        Ok(Some(_)) => HttpResponse::Ok().body("Device removed successfully"),
        Ok(None) => HttpResponse::NotFound().body("Device not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

pub async fn get_device_token(
    user_token: web::Json<String>,
) -> Option<String> {
    let user_token = user_token.into_inner();
    println!("user token: {}", user_token);

    // // 打印数据库中的所有数据
    // println!("数据库中的所有数据:");
    // for result in DB.iter() {
    //     match result {
    //         Ok((key, value)) => {
    //             let key_str = String::from_utf8_lossy(&key);
    //             let value_str = String::from_utf8_lossy(&value);
    //             println!("键: {}, 值: {}", key_str, value_str);
    //         },
    //         Err(e) => println!("遍历数据时出错: {}", e),
    //     }
    // }

    let result = DB
        .scan_prefix(user_token.as_bytes())
        .values()
        .next()
        .transpose(); // 处理可能的错误

    match result {
        Ok(Some(device_token_ivec)) => {
            match String::from_utf8(device_token_ivec.to_vec()) {
                Ok(device_token) => Some(device_token),
                Err(_) => None,
            }
        }
        Ok(None) => None,
        Err(_) => None,
    }
}