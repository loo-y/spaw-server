
## sPaw-server (Rust 服务器)

**简单推送 Webhook (sPaw) - 服务器**

此仓库包含使用 Rust 编写的 sPaw 的服务器组件，使用了 Actix Web 框架。它提供了一个简单的 API 端点，用于通过 Apple 推送通知服务 (APNs) 向 iOS 设备发送推送通知。

**功能：**

* 公开 `/pushmessage` 端点以接收消息和 APNs 令牌。
* 通过 APNs 向 iOS 设备发送推送通知。
* 易于部署和自托管。
* 开源且可定制。

**要求：**

* Rust 和 Cargo
* Apple 推送通知服务 (APNs) 密钥。请参阅 Apple 的文档以了解如何获取密钥。

**安装：**

1. 克隆此仓库：`git clone https://github.com/loo-y/spaw-server.git`
2. 导航到项目目录：`cd sPaw-server`
3. 构建服务器：`cargo build --release`
4. 运行服务器：`cargo run --release` （或使用 systemd 之类的进程管理器）

**配置：**

在运行服务器之前，您需要配置以下环境变量：

* `APNS_KEY_PATH`：APNs 密钥文件的路径。
* `APNS_KEY_ID`：您的 APNs 密钥 ID。
* `APNS_TEAM_ID`：您的 Apple 开发者团队 ID。
* `SERVER_PORT`（可选）：服务器应监听的端口。默认为 8080。

**API 端点：**

* `POST /pushmessage`

  * **请求正文：**
    ```json
    {
      "token": "您的设备令牌",
      "message": "您的消息文本"
    }
    ```

**部署：**

可以轻松地将服务器部署到任何支持 Rust 的 VPS。您可以使用 systemd 之类的进程管理器来管理服务器进程。