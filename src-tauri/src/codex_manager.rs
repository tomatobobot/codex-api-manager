use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as JsonValue};
use toml::Value as TomlValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ProfileType {
    #[default]
    Codex,
    Claude,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    #[serde(default)]
    pub profile_type: ProfileType,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppPaths {
    pub auth_json: PathBuf,
    pub config_toml: PathBuf,
    pub claude_settings_json: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveCodexValues {
    pub api_key: String,
    pub base_url: String,
    pub profile_type: ProfileType,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagerState {
    pub profiles: Vec<Profile>,
    pub active_codex_profile_id: Option<String>,
    pub active_claude_profile_id: Option<String>,
    pub codex_paths: ResolvedCodexPaths,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedCodexPaths {
    pub auth_json: String,
    pub config_toml: String,
    pub claude_settings_json: String,
}

pub fn profiles_file_path(app_config_dir: &Path) -> PathBuf {
    app_config_dir.join("profiles.json")
}

pub fn load_profiles_from_path(file_path: &Path) -> Result<Vec<Profile>, String> {
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(file_path)
        .map_err(|error| format!("读取账号列表失败: {error}"))?;

    serde_json::from_str(&raw).map_err(|error| format!("账号列表格式不正确: {error}"))
}

pub fn save_profiles_to_path(file_path: &Path, profiles: &[Profile]) -> Result<(), String> {
    validate_profiles(profiles)?;

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建账号列表目录失败: {error}"))?;
    }

    let content = serde_json::to_string_pretty(profiles)
        .map_err(|error| format!("生成账号列表内容失败: {error}"))?;

    fs::write(file_path, content).map_err(|error| format!("保存账号列表失败: {error}"))
}

pub fn resolve_codex_paths(os: &str, home_dir: &Path) -> Result<AppPaths, String> {
    let codex_dir = match os {
        "windows" | "macos" | "linux" => home_dir.join(".codex"),
        other => {
            return Err(format!("暂不支持的系统: {other}"));
        }
    };
    let claude_settings = home_dir.join(".claude").join("settings.json");

    Ok(AppPaths {
        auth_json: codex_dir.join("auth.json"),
        config_toml: codex_dir.join("config.toml"),
        claude_settings_json: claude_settings,
    })
}

pub fn resolve_runtime_codex_paths() -> Result<AppPaths, String> {
    let os = env::consts::OS;
    let home_dir = match os {
        "windows" => env::var_os("USERPROFILE")
            .or_else(|| {
                let drive = env::var_os("HOMEDRIVE")?;
                let path = env::var_os("HOMEPATH")?;
                Some(format!("{}{}", drive.to_string_lossy(), path.to_string_lossy()).into())
            })
            .ok_or_else(|| "找不到 Windows 用户目录".to_string())?,
        "macos" | "linux" => env::var_os("HOME").ok_or_else(|| "找不到用户目录".to_string())?,
        other => return Err(format!("暂不支持的系统: {other}")),
    };

    resolve_codex_paths(os, Path::new(&home_dir))
}

pub fn apply_profile_to_paths(
    profile: &Profile,
    auth_path: &Path,
    config_path: &Path,
    claude_settings_path: &Path,
) -> Result<(), String> {
    match profile.profile_type {
        ProfileType::Codex => apply_codex_profile(profile, auth_path, config_path),
        ProfileType::Claude => apply_claude_profile(profile, claude_settings_path),
    }
}

fn apply_codex_profile(
    profile: &Profile,
    auth_path: &Path,
    config_path: &Path,
) -> Result<(), String> {
    let auth_source = if auth_path.exists() {
        fs::read_to_string(auth_path).map_err(|error| format!("读取 auth.json 失败: {error}"))?
    } else {
        "{}".to_string()
    };

    let config_source = if config_path.exists() {
        fs::read_to_string(config_path)
            .map_err(|error| format!("读取 config.toml 失败: {error}"))?
    } else {
        String::new()
    };

    let auth_content = update_auth_json_str(&auth_source, &profile.api_key)?;
    let config_content = update_config_toml_str(&config_source, &profile.base_url)?;

    if let Some(parent) = auth_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建 auth.json 目录失败: {error}"))?;
    }
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建 config.toml 目录失败: {error}"))?;
    }

    fs::write(auth_path, auth_content).map_err(|error| format!("写入 auth.json 失败: {error}"))?;
    fs::write(config_path, config_content)
        .map_err(|error| format!("写入 config.toml 失败: {error}"))?;
    Ok(())
}

fn apply_claude_profile(profile: &Profile, settings_path: &Path) -> Result<(), String> {
    let source = if settings_path.exists() {
        fs::read_to_string(settings_path)
            .map_err(|error| format!("读取 settings.json 失败: {error}"))?
    } else {
        "{}".to_string()
    };

    let parsed: JsonValue = if source.trim().is_empty() {
        JsonValue::Object(Map::new())
    } else {
        serde_json::from_str(&source)
            .map_err(|error| format!("settings.json 格式不正确: {error}"))?
    };

    let mut root = match parsed {
        JsonValue::Object(map) => map,
        _ => return Err("settings.json 顶层必须是对象".to_string()),
    };

    let env_obj = root
        .entry("env")
        .or_insert_with(|| JsonValue::Object(Map::new()));
    let env_map = match env_obj {
        JsonValue::Object(map) => map,
        _ => return Err("settings.json 的 env 字段必须是对象".to_string()),
    };
    env_map.insert(
        "ANTHROPIC_AUTH_TOKEN".to_string(),
        JsonValue::String(profile.api_key.clone()),
    );
    env_map.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        JsonValue::String(profile.base_url.clone()),
    );

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建 settings.json 目录失败: {error}"))?;
    }

    let content = serde_json::to_string_pretty(&JsonValue::Object(root))
        .map_err(|error| format!("生成 settings.json 内容失败: {error}"))?;
    fs::write(settings_path, content)
        .map_err(|error| format!("写入 settings.json 失败: {error}"))?;
    Ok(())
}

pub fn match_active_profile(
    profiles: &[Profile],
    active: &ActiveCodexValues,
) -> Option<String> {
    profiles
        .iter()
        .find(|profile| {
            profile.profile_type == active.profile_type
                && profile.api_key == active.api_key
                && profile.base_url == active.base_url
        })
        .map(|profile| profile.id.clone())
}

pub fn resolve_active_profile_ids(
    profiles: &[Profile],
    paths: &AppPaths,
) -> Result<(Option<String>, Option<String>), String> {
    let codex_active = if paths.auth_json.exists() && paths.config_toml.exists() {
        let auth_source = fs::read_to_string(&paths.auth_json)
            .map_err(|e| format!("读取 auth.json 失败: {e}"))?;
        let config_source = fs::read_to_string(&paths.config_toml)
            .map_err(|e| format!("读取 config.toml 失败: {e}"))?;
        let auth_json: JsonValue = serde_json::from_str(&auth_source)
            .map_err(|e| format!("auth.json 格式不正确: {e}"))?;
        let config_toml: TomlValue = config_source
            .parse()
            .map_err(|e| format!("config.toml 格式不正确: {e}"))?;
        let api_key = auth_json.get("OPENAI_API_KEY").and_then(JsonValue::as_str);
        let base_url = config_toml.get("base_url").and_then(TomlValue::as_str);
        if let (Some(k), Some(u)) = (api_key, base_url) {
            Some(ActiveCodexValues { api_key: k.to_string(), base_url: u.to_string(), profile_type: ProfileType::Codex })
        } else { None }
    } else { None };

    let claude_active = if paths.claude_settings_json.exists() {
        if let Ok(source) = fs::read_to_string(&paths.claude_settings_json) {
            if let Ok(JsonValue::Object(root)) = serde_json::from_str::<JsonValue>(&source) {
                if let Some(JsonValue::Object(env)) = root.get("env") {
                    let token = env.get("ANTHROPIC_AUTH_TOKEN").and_then(JsonValue::as_str);
                    let base_url = env.get("ANTHROPIC_BASE_URL").and_then(JsonValue::as_str);
                    if let (Some(t), Some(u)) = (token, base_url) {
                        Some(ActiveCodexValues { api_key: t.to_string(), base_url: u.to_string(), profile_type: ProfileType::Claude })
                    } else { None }
                } else { None }
            } else { None }
        } else { None }
    } else { None };

    let active_codex_id = codex_active.as_ref().and_then(|a| match_active_profile(profiles, a));
    let active_claude_id = claude_active.as_ref().and_then(|a| match_active_profile(profiles, a));
    Ok((active_codex_id, active_claude_id))
}

pub fn build_manager_state(
    profiles: Vec<Profile>,
    codex_paths: AppPaths,
) -> Result<ManagerState, String> {
    let (active_codex_profile_id, active_claude_profile_id) =
        resolve_active_profile_ids(&profiles, &codex_paths)?;

    Ok(ManagerState {
        profiles,
        active_codex_profile_id,
        active_claude_profile_id,
        codex_paths: ResolvedCodexPaths {
            auth_json: codex_paths.auth_json.to_string_lossy().into_owned(),
            config_toml: codex_paths.config_toml.to_string_lossy().into_owned(),
            claude_settings_json: codex_paths.claude_settings_json.to_string_lossy().into_owned(),
        },
    })
}

pub fn build_manager_state_for_local_save(
    profiles: Vec<Profile>,
    codex_paths: AppPaths,
) -> ManagerState {
    let (active_codex_profile_id, active_claude_profile_id) =
        resolve_active_profile_ids(&profiles, &codex_paths).unwrap_or((None, None));

    ManagerState {
        profiles,
        active_codex_profile_id,
        active_claude_profile_id,
        codex_paths: ResolvedCodexPaths {
            auth_json: codex_paths.auth_json.to_string_lossy().into_owned(),
            config_toml: codex_paths.config_toml.to_string_lossy().into_owned(),
            claude_settings_json: codex_paths.claude_settings_json.to_string_lossy().into_owned(),
        },
    }
}

fn validate_profiles(profiles: &[Profile]) -> Result<(), String> {
    let mut ids = std::collections::HashSet::new();

    for profile in profiles {
        if profile.id.trim().is_empty() {
            return Err("账号缺少 id".to_string());
        }
        if profile.name.trim().is_empty() {
            return Err("账号名称不能为空".to_string());
        }
        if profile.api_key.trim().is_empty() {
            return Err("API Key 不能为空".to_string());
        }
        if profile.base_url.trim().is_empty() {
            return Err("Base URL 不能为空".to_string());
        }
        if !ids.insert(profile.id.clone()) {
            return Err("账号 id 不能重复".to_string());
        }
    }

    Ok(())
}

fn update_auth_json_str(input: &str, api_key: &str) -> Result<String, String> {
    let parsed = if input.trim().is_empty() {
        JsonValue::Object(Map::new())
    } else {
        serde_json::from_str(input).map_err(|error| format!("auth.json 格式不正确: {error}"))?
    };

    let mut object = match parsed {
        JsonValue::Object(map) => map,
        _ => return Err("auth.json 顶层必须是对象".to_string()),
    };

    object.insert(
        "OPENAI_API_KEY".to_string(),
        JsonValue::String(api_key.to_string()),
    );

    serde_json::to_string_pretty(&JsonValue::Object(object))
        .map_err(|error| format!("生成 auth.json 内容失败: {error}"))
}

fn update_config_toml_str(input: &str, base_url: &str) -> Result<String, String> {
    let parsed = if input.trim().is_empty() {
        TomlValue::Table(toml::Table::new())
    } else {
        input
            .parse::<TomlValue>()
            .map_err(|error| format!("config.toml 格式不正确: {error}"))?
    };

    let mut table = match parsed {
        TomlValue::Table(table) => table,
        _ => return Err("config.toml 顶层必须是表".to_string()),
    };

    table.insert(
        "base_url".to_string(),
        TomlValue::String(base_url.to_string()),
    );

    toml::to_string(&table).map_err(|error| format!("生成 config.toml 内容失败: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn saves_and_loads_profiles() {
        let file_path = unique_temp_file("profiles.json");
        let profiles = vec![Profile {
            id: "primary".into(),
            name: "Primary".into(),
            api_key: "sk-primary".into(),
            base_url: "https://api.example.com".into(),
            profile_type: ProfileType::Codex,
        }];

        save_profiles_to_path(&file_path, &profiles).unwrap();

        let saved = load_profiles_from_path(&file_path).unwrap();
        assert_eq!(saved, profiles);
    }

    #[test]
    fn resolves_platform_specific_codex_paths() {
        let mac_paths = resolve_codex_paths("macos", Path::new("/Users/demo")).unwrap();
        assert_eq!(mac_paths.auth_json, PathBuf::from("/Users/demo/.codex/auth.json"));
        assert_eq!(mac_paths.config_toml, PathBuf::from("/Users/demo/.codex/config.toml"));
        assert_eq!(mac_paths.claude_settings_json, PathBuf::from("/Users/demo/.claude/settings.json"));

        let windows_paths = resolve_codex_paths("windows", Path::new("C:\\Users\\demo")).unwrap();
        assert_eq!(
            windows_paths.auth_json,
            PathBuf::from("C:\\Users\\demo\\.codex\\auth.json")
        );
        assert_eq!(
            windows_paths.config_toml,
            PathBuf::from("C:\\Users\\demo\\.codex\\config.toml")
        );
        assert_eq!(
            windows_paths.claude_settings_json,
            PathBuf::from("C:\\Users\\demo\\.claude\\settings.json")
        );
    }

    #[test]
    fn updates_auth_json_and_config_toml_together() {
        let auth_path = unique_temp_file("auth.json");
        let config_path = unique_temp_file("config.toml");
        fs::write(&auth_path, "{\n  \"OPENAI_API_KEY\": \"old-key\"\n}").unwrap();
        fs::write(&config_path, "base_url = \"https://old.example.com\"\n").unwrap();

        let profile = Profile {
            id: "secondary".into(),
            name: "Secondary".into(),
            api_key: "sk-secondary".into(),
            base_url: "https://new.example.com".into(),
            profile_type: ProfileType::Codex,
        };

        apply_profile_to_paths(&profile, &auth_path, &config_path, &unique_temp_file("settings.json")).unwrap();

        let auth_text = fs::read_to_string(&auth_path).unwrap();
        let config_text = fs::read_to_string(&config_path).unwrap();
        assert!(auth_text.contains("\"OPENAI_API_KEY\": \"sk-secondary\""));
        assert!(config_text.contains("base_url = \"https://new.example.com\""));
    }

    #[test]
    fn matches_the_current_profile_from_live_values() {
        let profiles = vec![
            Profile {
                id: "first".into(),
                name: "First".into(),
                api_key: "sk-first".into(),
                base_url: "https://first.example.com".into(),
                profile_type: ProfileType::Codex,
            },
            Profile {
                id: "second".into(),
                name: "Second".into(),
                api_key: "sk-second".into(),
                base_url: "https://second.example.com".into(),
                profile_type: ProfileType::Codex,
            },
        ];

        let active = ActiveCodexValues {
            api_key: "sk-second".into(),
            base_url: "https://second.example.com".into(),
            profile_type: ProfileType::Codex,
        };

        assert_eq!(match_active_profile(&profiles, &active), Some("second".into()));
    }

    #[test]
    fn local_save_state_does_not_fail_when_config_is_incomplete() {
        let profiles = vec![Profile {
            id: "first".into(),
            name: "First".into(),
            api_key: "sk-first".into(),
            base_url: "https://first.example.com".into(),
            profile_type: ProfileType::Codex,
        }];
        let auth_path = unique_temp_file("auth.json");
        let config_path = unique_temp_file("config.toml");
        fs::write(&auth_path, "{\n  \"OPENAI_API_KEY\": \"sk-first\"\n}").unwrap();
        fs::write(&config_path, "model = \"gpt-5\"\n").unwrap();

        let state = build_manager_state_for_local_save(
            profiles.clone(),
            AppPaths {
                auth_json: auth_path,
                config_toml: config_path,
                claude_settings_json: unique_temp_file("settings.json"),
            },
        );

        assert_eq!(state.profiles, profiles);
        assert_eq!(state.active_codex_profile_id, None);
        assert_eq!(state.active_claude_profile_id, None);
    }

    fn unique_temp_file(file_name: &str) -> PathBuf {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("codex-api-manager-{unique_id}"));
        fs::create_dir_all(&dir).unwrap();
        dir.join(file_name)
    }
}
