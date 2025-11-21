use axum::extract::Query;
use axum::routing::get;
use tokio::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tauri::{Manager as tauri_manager, async_runtime, AppHandle, Emitter, Wry};

static APP_HANDLE: OnceCell<AppHandle<Wry>> = OnceCell::const_new();

pub fn start_web_server(app_handle: AppHandle){
    APP_HANDLE.set(app_handle).unwrap();

    async_runtime::spawn(async move {
        // 启动 Axum HTTP 服务
        let app = axum::Router::new()
            .route("/download", get(download))
            .route("/", get(hello));
        let listener_res = tokio::net::TcpListener::bind("0.0.0.0:56881").await;
        log::info!("启动webServer port 56881");
        if listener_res.is_err() {
            log::error!("启动webServer port 失败: {}", listener_res.unwrap_err());
            return;
        }
        log::info!("启动webServer 成功");
        let res = axum::serve(listener_res.unwrap(), app).await;
        if res.is_err() {
            log::error!("启动webServer失败: {}", res.unwrap_err());
        }
    });
}
async fn hello() -> &'static str {
    "Hello from Tauri HTTP Server"
}
#[derive(Deserialize, Debug)]
struct DownloadQuery{
    url: String,
    title: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct DownloadPayload {
    url: String,
    title: Option<String>,
}
async fn download(Query(query): Query<DownloadQuery>) -> impl axum::response::IntoResponse {
    log::info!("query：{:?}", query);
    if let Some(app_handle) = APP_HANDLE.get() {
        let payload = DownloadPayload {
            url: query.url,
            title: query.title,
        };
        if let Err(e) = app_handle.emit("new-download-task", payload) {
            log::error!("emit new-download-task event fail: {}", e);
        }
        if let Some(window) = app_handle.get_webview_window("main") {
            if let Err(e) = window.show() {
                log::error!("Failed to show window: {}", e);
            }
            if let Err(e) = window.set_focus() {
                log::error!("Failed to focus window: {}", e);
            }
        }
    }
    (
        [("Access-Control-Allow-Origin", "*")],
        "ok"
    )
}