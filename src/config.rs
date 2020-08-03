use crate::{
    direction::Direction,
    keybindings::{key_press::KeyPress, keybinding::Keybinding, keybinding_type::KeybindingType},
    split_direction::SplitDirection,
};
use log::{debug, error};
use regex::Regex;
use std::io::Write;
use std::str::FromStr;
use winapi::um::wingdi::GetBValue;
use winapi::um::wingdi::GetGValue;
use winapi::um::wingdi::GetRValue;
use winapi::um::wingdi::RGB;

#[macro_use]
mod macros;

pub mod hot_reloading;

#[derive(Debug, Clone)]
pub struct Rule {
    pub pattern: Regex,
    pub has_custom_titlebar: bool,
    pub manage: bool,
    pub chromium: bool,
    pub firefox: bool,
    pub remove_frame: bool,
    pub workspace: i32,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            pattern: Regex::new("").unwrap(),
            has_custom_titlebar: false,
            manage: true,
            remove_frame: true,
            chromium: false,
            firefox: false,
            workspace: -1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceSetting {
    pub id: i32,
    pub monitor: i32,
}

impl Default for WorkspaceSetting {
    fn default() -> Self {
        Self {
            id: -1,
            monitor: -1,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub app_bar_height: i32,
    pub app_bar_bg: i32,
    pub app_bar_font: String,
    pub app_bar_date_pattern: String,
    pub app_bar_time_pattern: String,
    pub use_border: bool,
    pub app_bar_font_size: i32,
    pub min_width: i32,
    pub min_height: i32,
    pub work_mode: bool,
    pub light_theme: bool,
    pub multi_monitor: bool,
    pub launch_on_startup: bool,
    pub margin: i32,
    pub padding: i32,
    pub remove_title_bar: bool,
    pub remove_task_bar: bool,
    pub display_app_bar: bool,
    pub workspace_settings: Vec<WorkspaceSetting>,
    pub keybindings: Vec<Keybinding>,
    pub rules: Vec<Rule>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_bar_height: 20,
            app_bar_bg: 0x2e3440,
            app_bar_font: String::from("Consolas"),
            app_bar_font_size: 18,
            app_bar_date_pattern: String::from("%e %b %Y"),
            app_bar_time_pattern: String::from("%T"),
            launch_on_startup: false,
            min_height: 0,
            min_width: 0,
            use_border: false,
            margin: 0,
            padding: 0,
            remove_title_bar: false,
            work_mode: true,
            light_theme: false,
            multi_monitor: false,
            remove_task_bar: false,
            display_app_bar: false,
            workspace_settings: Vec::new(),
            keybindings: Vec::new(),
            rules: Vec::new(),
        }
    }
}

impl Config {
    /// Creates a new default config.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_field(self: &mut Self, field: &str, value: i32) {
        self.alter_numerical_field(field, value);
    }

    pub fn decrement_field(self: &mut Self, field: &str, value: i32) {
        self.alter_numerical_field(field, -value);
    }

    fn alter_numerical_field(self: &mut Self, field: &str, value: i32) {
        match field {
            "app_bar_height" => self.app_bar_height += value,
            "app_bar_bg" => self.app_bar_bg += value,
            "app_bar_font_size" => self.app_bar_font_size += value,
            "margin" => self.margin += value,
            "padding" => self.padding += value,
            _ => error!("Attempt to alter unknown field: {} by {}", field, value),
        }
    }

    pub fn toggle_field(self: &mut Self, field: &str) {
        match field {
            "use_border" => self.use_border = !self.use_border,
            "light_theme" => self.light_theme = !self.light_theme,
            "launch_on_startup" => self.launch_on_startup = !self.launch_on_startup,
            "remove_title_bar" => self.remove_title_bar = !self.remove_title_bar,
            "remove_task_bar" => self.remove_task_bar = !self.remove_task_bar,
            "display_app_bar" => self.display_app_bar = !self.display_app_bar,
            _ => error!("Attempt to toggle unknown field: {}", field),
        }
    }
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    let mut pathbuf = match dirs::config_dir() {
        Some(path) => path,
        None => std::path::PathBuf::new(),
    };

    pathbuf.push("wwm");

    if !pathbuf.exists() {
        debug!("wwm folder doesn't exist yet. Creating the folder");
        std::fs::create_dir(pathbuf.clone())?;
    }

    pathbuf.push("config.yaml");

    if !pathbuf.exists() {
        debug!("config file doesn't exist yet. Creating the file");
        if let Ok(mut file) = std::fs::File::create(pathbuf.clone()) {
            debug!("Initializing config with default values");
            file.write_all(include_bytes!("../default_config.yaml"))?;
        }
    }

    let path = match pathbuf.to_str() {
        Some(string) => string,
        None => "",
    };

    let file_content = std::fs::read_to_string(path)?;

    let vec_yaml = yaml_rust::YamlLoader::load_from_str(&file_content)?;
    let yaml = if !vec_yaml.is_empty() {
        &vec_yaml[0]
    } else {
        &yaml_rust::Yaml::Null
    };

    let mut config = Config::new();

    if let yaml_rust::yaml::Yaml::Hash(hash) = yaml {
        for entry in hash.iter() {
            let (key, value) = entry;
            let config_key = key.as_str().ok_or("Invalid config key")?;

            if_str!(config, config_key, value, app_bar_font);
            if_str!(config, config_key, value, app_bar_date_pattern);
            if_str!(config, config_key, value, app_bar_time_pattern);
            if_i32!(config, config_key, value, app_bar_bg);
            if_i32!(config, config_key, value, app_bar_font_size);
            if_i32!(config, config_key, value, app_bar_height);
            if_i32!(config, config_key, value, min_width);
            if_i32!(config, config_key, value, min_height);
            if_i32!(config, config_key, value, margin);
            if_i32!(config, config_key, value, padding);
            if_bool!(config, config_key, value, use_border);
            if_bool!(config, config_key, value, light_theme);
            if_bool!(config, config_key, value, launch_on_startup);
            if_bool!(config, config_key, value, work_mode);
            if_bool!(config, config_key, value, multi_monitor);
            if_bool!(config, config_key, value, remove_title_bar);
            if_bool!(config, config_key, value, remove_task_bar);
            if_bool!(config, config_key, value, display_app_bar);

            if config_key == "workspaces" {
                let workspaces = value.as_vec().ok_or("workspaces has to be an array")?;

                for yaml_workspace in workspaces {
                    if let yaml_rust::Yaml::Hash(hash) = yaml_workspace {
                        let mut workspace = WorkspaceSetting::default();

                        for entry in hash.iter() {
                            let (key, value) = entry;
                            let hash_key = key.as_str().ok_or("Invalid config key")?;

                            if_i32!(workspace, hash_key, value, id);
                            if_i32!(workspace, hash_key, value, monitor);
                        }

                        config.workspace_settings.push(workspace);
                    }
                }
            }

            if config_key == "rules" {
                let rules = value.as_vec().ok_or("rules has to be an array")?;

                for yaml_rule in rules {
                    if let yaml_rust::Yaml::Hash(hash) = yaml_rule {
                        let mut rule = Rule::default();

                        for entry in hash.iter() {
                            let (key, value) = entry;
                            let hash_key = key.as_str().ok_or("Invalid config key")?;

                            if_regex!(rule, hash_key, value, pattern);
                            if_bool!(rule, hash_key, value, has_custom_titlebar);
                            if_bool!(rule, hash_key, value, remove_frame);
                            if_bool!(rule, hash_key, value, manage);
                            if_bool!(rule, hash_key, value, chromium);
                            if_bool!(rule, hash_key, value, firefox);
                            if_i32!(rule, hash_key, value, workspace);
                        }

                        config.rules.push(rule);
                    }
                }
            }

            if config_key == "keybindings" {
                let bindings = value.as_vec().ok_or("keybindings has to be an array")?;

                for binding in bindings {
                    let typ_str = ensure_str!("keybinding", binding, type);
                    let key_press = KeyPress::from_str(ensure_str!("keybinding", binding, key))?;

                    let maybe_typ =
                        match typ_str {
                            "Launch" => Some(KeybindingType::Launch(
                                ensure_str!("keybinding of type Launch", binding, cmd).to_string(),
                            )),
                            "CloseTile" => Some(KeybindingType::CloseTile),
                            "Quit" => Some(KeybindingType::Quit),
                            "ChangeWorkspace" => Some(KeybindingType::ChangeWorkspace(
                                ensure_i32!("keybinding of type ChangeWorkspace", binding, id),
                            )),
                            "MoveToWorkspace" => Some(KeybindingType::MoveToWorkspace(
                                ensure_i32!("keybinding of type MoveToWorkspace", binding, id),
                            )),
                            "MoveWorkspaceToMonitor" => {
                                Some(KeybindingType::MoveWorkspaceToMonitor(ensure_i32!(
                                    "keybinding of type MoveWorkspaceToMonitor",
                                    binding,
                                    monitor
                                )))
                            }
                            "ToggleFloatingMode" => Some(KeybindingType::ToggleFloatingMode),
                            "ToggleFullscreen" => Some(KeybindingType::ToggleFullscreen),
                            "ToggleWorkMode" => Some(KeybindingType::ToggleWorkMode),
                            "IncrementConfig" => Some(KeybindingType::IncrementConfig(
                                ensure_str!("keybinding of type IncrementConfig", binding, field)
                                    .to_string(),
                                ensure_i32!("keybinding of type IncrementConfig", binding, value),
                            )),
                            "DecrementConfig" => Some(KeybindingType::DecrementConfig(
                                ensure_str!("keybinding of type DecrementConfig", binding, field)
                                    .to_string(),
                                ensure_i32!("keybinding of type DecrementConfig", binding, value),
                            )),
                            "ToggleConfig" => Some(KeybindingType::ToggleConfig(
                                ensure_str!("keybinding of type ToggleConfig", binding, field)
                                    .to_string(),
                            )),
                            "Focus" => Some(KeybindingType::Focus(Direction::from_str(
                                ensure_str!("keybinding of type Focus", binding, direction),
                            )?)),
                            "Resize" => Some(KeybindingType::Resize(
                                Direction::from_str(ensure_str!(
                                    "keybinding of type Resize",
                                    binding,
                                    direction
                                ))?,
                                ensure_i32!("keybinding of type Resize", binding, amount),
                            )),
                            "Swap" => Some(KeybindingType::Swap(Direction::from_str(
                                ensure_str!("keybinding of type Swap", binding, direction),
                            )?)),
                            "Split" => Some(KeybindingType::Split(SplitDirection::from_str(
                                ensure_str!("keybinding of type Split", binding, direction),
                            )?)),
                            x => {
                                error!("unknown keybinding type {}", x);
                                None
                            }
                        };

                    if let Some(typ) = maybe_typ {
                        let mut kb = Keybinding::from(key_press);
                        kb.typ = typ;
                        config.keybindings.push(kb);
                    }
                }
            }
        }
        //Convert normal hexadecimal color format to winapi hexadecimal color format
        convert_color_format!(config.app_bar_bg);
    }
    Ok(config)
}
