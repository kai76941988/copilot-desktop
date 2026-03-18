use crate::app::window::open_memory_hub_window;
use crate::memory::{
    db as memory_db, MemoryCreateProjectParams, MemoryGetContextPackParams,
    MemoryContinueParams, MemoryListMessagesParams, MemoryListSessionsParams,
    MemoryListSummariesParams, MemoryProjectInfo, MemoryRecordMessageParams, MemorySearchParams,
    MemorySearchItem, MemorySearchSummariesParams, MemorySessionInfo, MemorySetProjectParams,
    MemorySummaryInfo, MemoryMessageInfo, SummarizerConfig, MemoryRefreshSummariesParams,
    MemorySetMessageTagsParams, MemoryAutoTagParams, MemoryTagInfo, MemoryExportInfo,
};
use serde_json::json;
use crate::util::{check_file_or_append, get_download_message_with_lang, show_toast, MessageType};
use std::fs::{self, File};
use std::io::Write;
use std::str::FromStr;
use tauri::http::Method;
use tauri::{command, AppHandle, Manager, Url, WebviewWindow};
use tauri_plugin_http::reqwest::{ClientBuilder, Request};

#[cfg(target_os = "macos")]
use tauri::Theme;

#[derive(serde::Deserialize)]
pub struct DownloadFileParams {
    url: String,
    filename: String,
    language: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct BinaryDownloadParams {
    filename: String,
    binary: Vec<u8>,
    language: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct NotificationParams {
    title: String,
    body: String,
    icon: String,
}

#[command]
pub async fn download_file(app: AppHandle, params: DownloadFileParams) -> Result<(), String> {
    let window: WebviewWindow = app.get_webview_window("pake").ok_or("Window not found")?;

    show_toast(
        &window,
        &get_download_message_with_lang(MessageType::Start, params.language.clone()),
    );

    let download_dir = app
        .path()
        .download_dir()
        .map_err(|e| format!("Failed to get download dir: {}", e))?;

    let output_path = download_dir.join(&params.filename);

    let path_str = output_path.to_str().ok_or("Invalid output path")?;

    let file_path = check_file_or_append(path_str);

    let client = ClientBuilder::new()
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let url = Url::from_str(&params.url).map_err(|e| format!("Invalid URL: {}", e))?;

    let request = Request::new(Method::GET, url);

    let response = client.execute(request).await;

    match response {
        Ok(mut res) => {
            let mut file =
                File::create(file_path).map_err(|e| format!("Failed to create file: {}", e))?;

            while let Some(chunk) = res
                .chunk()
                .await
                .map_err(|e| format!("Failed to get chunk: {}", e))?
            {
                file.write_all(&chunk)
                    .map_err(|e| format!("Failed to write chunk: {}", e))?;
            }

            show_toast(
                &window,
                &get_download_message_with_lang(MessageType::Success, params.language.clone()),
            );
            Ok(())
        }
        Err(e) => {
            show_toast(
                &window,
                &get_download_message_with_lang(MessageType::Failure, params.language),
            );
            Err(e.to_string())
        }
    }
}

#[command]
pub async fn download_file_by_binary(
    app: AppHandle,
    params: BinaryDownloadParams,
) -> Result<(), String> {
    let window: WebviewWindow = app.get_webview_window("pake").ok_or("Window not found")?;

    show_toast(
        &window,
        &get_download_message_with_lang(MessageType::Start, params.language.clone()),
    );

    let download_dir = app
        .path()
        .download_dir()
        .map_err(|e| format!("Failed to get download dir: {}", e))?;

    let output_path = download_dir.join(&params.filename);

    let path_str = output_path.to_str().ok_or("Invalid output path")?;

    let file_path = check_file_or_append(path_str);

    match fs::write(file_path, &params.binary) {
        Ok(_) => {
            show_toast(
                &window,
                &get_download_message_with_lang(MessageType::Success, params.language.clone()),
            );
            Ok(())
        }
        Err(e) => {
            show_toast(
                &window,
                &get_download_message_with_lang(MessageType::Failure, params.language),
            );
            Err(e.to_string())
        }
    }
}

#[command]
pub fn memory_record_message(app: AppHandle, params: MemoryRecordMessageParams) -> Result<(), String> {
    memory_db::record_message(&app, params)
}

#[command]
pub fn memory_create_project(
    app: AppHandle,
    params: MemoryCreateProjectParams,
) -> Result<String, String> {
    memory_db::create_project(&app, params)
}

#[command]
pub fn memory_list_projects(app: AppHandle) -> Result<Vec<MemoryProjectInfo>, String> {
    memory_db::list_projects(&app)
}

#[command]
pub fn memory_list_sessions(
    app: AppHandle,
    params: MemoryListSessionsParams,
) -> Result<Vec<MemorySessionInfo>, String> {
    memory_db::list_sessions(&app, params)
}

#[command]
pub fn memory_get_context_pack(
    app: AppHandle,
    params: MemoryGetContextPackParams,
) -> Result<String, String> {
    memory_db::get_context_pack(&app, params)
}

#[command]
pub fn memory_search_messages(
    app: AppHandle,
    params: MemorySearchParams,
) -> Result<Vec<MemorySearchItem>, String> {
    memory_db::search_messages(&app, params)
}

#[command]
pub fn memory_search_summaries(
    app: AppHandle,
    params: MemorySearchSummariesParams,
) -> Result<Vec<MemorySummaryInfo>, String> {
    memory_db::search_summaries(&app, params)
}

#[command]
pub fn memory_continue_project(app: AppHandle, params: MemoryContinueParams) -> Result<String, String> {
    let pack = memory_db::get_context_pack(
        &app,
        MemoryGetContextPackParams {
            project_id: params.project_id.clone(),
            session_id: params.session_id.clone(),
        },
    )?;

    if let Some(window) = app.get_webview_window("pake") {
        if let Some(project_id) = params.project_id.clone() {
            let _ = window.emit("memory_set_project", json!({ "project_id": project_id }));
        }
        let _ = window.show();
        let _ = window.set_focus();
        let payload = json!({
            "text": pack,
            "forceNewChat": params.open_new.unwrap_or(true)
        });
        window
            .emit("memory_context_pack", payload)
            .map_err(|e| format!("Emit context pack failed: {}", e))?;
    }

    Ok(pack)
}

#[command]
pub fn memory_set_active_project(
    app: AppHandle,
    params: MemorySetProjectParams,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("pake") {
        window
            .emit("memory_set_project", json!({ "project_id": params.project_id }))
            .map_err(|e| format!("Emit set project failed: {}", e))?;
    }
    Ok(())
}

#[command]
pub fn memory_list_summaries(
    app: AppHandle,
    params: MemoryListSummariesParams,
) -> Result<Vec<MemorySummaryInfo>, String> {
    memory_db::list_summaries(&app, params)
}

#[command]
pub fn memory_list_messages(
    app: AppHandle,
    params: MemoryListMessagesParams,
) -> Result<Vec<MemoryMessageInfo>, String> {
    memory_db::list_messages(&app, params)
}

#[command]
pub fn memory_get_summarizer_config(app: AppHandle) -> Result<SummarizerConfig, String> {
    memory_db::get_summarizer_config(&app)
}

#[command]
pub fn memory_set_summarizer_config(
    app: AppHandle,
    params: SummarizerConfig,
) -> Result<(), String> {
    memory_db::set_summarizer_config(&app, params)
}

#[command]
pub async fn memory_refresh_summaries(
    app: AppHandle,
    params: MemoryRefreshSummariesParams,
) -> Result<(), String> {
    memory_db::refresh_summaries(&app, params).await
}

#[command]
pub fn memory_set_message_tags(
    app: AppHandle,
    params: MemorySetMessageTagsParams,
) -> Result<(), String> {
    memory_db::set_message_tags(&app, params)
}

#[command]
pub fn memory_auto_tag_messages(
    app: AppHandle,
    params: MemoryAutoTagParams,
) -> Result<i64, String> {
    memory_db::auto_tag_messages(&app, params)
}

#[command]
pub fn memory_list_tags(app: AppHandle) -> Result<Vec<MemoryTagInfo>, String> {
    memory_db::list_tags(&app)
}

#[command]
pub fn memory_export_project(
    app: AppHandle,
    project_id: Option<String>,
) -> Result<MemoryExportInfo, String> {
    memory_db::export_project_jsonl(&app, project_id)
}

#[command]
pub fn memory_open_hub(app: AppHandle) -> Result<(), String> {
    open_memory_hub_window(&app).map(|_| ()).map_err(|e| e.to_string())
}

#[command]
pub fn send_notification(app: AppHandle, params: NotificationParams) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title(&params.title)
        .body(&params.body)
        .icon(&params.icon)
        .show()
        .map_err(|e| format!("Failed to show notification: {}", e))?;
    Ok(())
}

#[command]
pub async fn update_theme_mode(app: AppHandle, mode: String) {
    #[cfg(target_os = "macos")]
    {
        if let Some(window) = app.get_webview_window("pake") {
            let theme = if mode == "dark" {
                Theme::Dark
            } else {
                Theme::Light
            };
            let _ = window.set_theme(Some(theme));
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
        let _ = mode;
    }
}

#[command]
#[allow(unreachable_code)]
pub fn clear_cache_and_restart(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("pake") {
        match window.clear_all_browsing_data() {
            Ok(_) => {
                // Clear all browsing data successfully
                app.restart();
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to clear browsing data: {}", e);
                Err(format!("Failed to clear browsing data: {}", e))
            }
        }
    } else {
        Err("Main window not found".to_string())
    }
}
