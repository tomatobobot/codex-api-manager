mod codex_manager;

use codex_manager::{
    apply_profile_to_paths, build_manager_state, build_manager_state_for_local_save,
    load_profiles_from_path, profiles_file_path, resolve_runtime_codex_paths, save_profiles_to_path,
    ManagerState, Profile,
};
use tauri::{AppHandle, Manager};

/// 加载完整的管理器状态：从应用配置目录读取账号列表，
/// 并读取当前系统的 Codex / Claude 配置文件来确定哪个账号正在生效。
#[tauri::command]
fn load_manager_state(app: AppHandle) -> Result<ManagerState, String> {
    let app_config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("找不到应用配置目录: {error}"))?;
    let profiles_path = profiles_file_path(&app_config_dir);
    let profiles = load_profiles_from_path(&profiles_path)?;
    let codex_paths = resolve_runtime_codex_paths()?;
    build_manager_state(profiles, codex_paths)
}

/// 保存账号列表到应用配置目录（不修改 Codex / Claude 配置文件）。
/// 返回的状态中 active_*_profile_id 尽力读取，失败时为 None。
#[tauri::command]
fn save_profiles(app: AppHandle, profiles: Vec<Profile>) -> Result<ManagerState, String> {
    let app_config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("找不到应用配置目录: {error}"))?;
    let profiles_path = profiles_file_path(&app_config_dir);
    save_profiles_to_path(&profiles_path, &profiles)?;
    let codex_paths = resolve_runtime_codex_paths()?;
    Ok(build_manager_state_for_local_save(profiles, codex_paths))
}

/// 切换到指定账号：将该账号的 API Key 和 Base URL 写入对应配置文件，
/// 然后重新读取状态返回给前端。
#[tauri::command]
fn activate_profile(app: AppHandle, profile_id: String) -> Result<ManagerState, String> {
    let app_config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("找不到应用配置目录: {error}"))?;
    let profiles_path = profiles_file_path(&app_config_dir);
    let profiles = load_profiles_from_path(&profiles_path)?;
    let selected = profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .cloned()
        .ok_or_else(|| "没有找到要切换的账号".to_string())?;
    let codex_paths = resolve_runtime_codex_paths()?;
    apply_profile_to_paths(&selected, &codex_paths.auth_json, &codex_paths.config_toml, &codex_paths.claude_settings_json)?;
    build_manager_state(profiles, codex_paths)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_manager_state,
            save_profiles,
            activate_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
