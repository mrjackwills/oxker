use std::{
    io::{Read, Write},
    path::PathBuf,
};

use serde::Deserialize;

use crate::app_error::AppError;

use super::{color_parser::ConfigColors, keymap_parser::ConfigKeymap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigFileType {
    Toml,
    Jsonc,
    Json,
    JsoncAsJson,
}

impl TryFrom<&PathBuf> for ConfigFileType {
    type Error = AppError;

    fn try_from(value: &PathBuf) -> Result<Self, AppError> {
        let err = || AppError::IO(format!("Can't parse give config file: {}", value.display()));
        let Some(ext) = value.extension() else {
            return Err(err());
        };
        let Some(ext) = ext.to_str() else {
            return Err(err());
        };
        match ext {
            "toml" => Ok(Self::Toml),
            "json" => Ok(Self::Json),
            "jsonc" => Ok(Self::Jsonc),
            _ => Err(err()),
        }
    }
}

impl ConfigFileType {
    /// Get the local config directory, to be used by default config parser
    fn get_config_dir(in_container: bool) -> Option<PathBuf> {
        if in_container {
            Some(PathBuf::from("/"))
        } else {
            directories::BaseDirs::new()
                .map(|base_dirs| base_dirs.config_local_dir().join(env!("CARGO_PKG_NAME")))
        }
    }
    // should take in a pathbuf as well?
    fn get_default_filename(self, in_container: bool) -> PathBuf {
        let suffix = match self {
            Self::Json | Self::JsoncAsJson => "config.json",
            Self::Jsonc => "config.jsonc",
            Self::Toml => "config.toml",
        };
        Self::get_config_dir(in_container).map_or_else(|| PathBuf::from(suffix), |i| i.join(suffix))
    }
}

// impl ConfigFileType

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ConfigFile {
    pub color_logs: Option<bool>,
    pub docker_interval: Option<u32>,
    pub gui: Option<bool>,
    pub host: Option<String>,
    pub raw_logs: Option<bool>,
    pub show_timestamp: Option<bool>,
    pub save_dir: Option<String>,
    pub show_self: Option<bool>,
    pub show_std_err: Option<bool>,
    pub use_cli: Option<bool>,
    pub colors: Option<ConfigColors>,
    pub keymap: Option<ConfigKeymap>,
}

impl ConfigFile {
    /// Attempt to create an example.config.toml file, will attempt to recursively create the directories as well
    fn create_example_file(in_container: bool) -> Result<(), AppError> {
        if in_container {
            return Ok(());
        }

        let config_dir = ConfigFileType::get_config_dir(in_container)
            .ok_or_else(|| AppError::IO("config_dir".to_owned()))?;
        let file_name = config_dir.join("example.config.toml");

        if !std::fs::exists(&file_name).map_err(|i| AppError::IO(i.to_string()))? {
            if !std::fs::exists(&config_dir).map_err(|i| AppError::IO(i.to_string()))? {
                std::fs::DirBuilder::new()
                    .recursive(true)
                    .create(&config_dir)
                    .map_err(|i| AppError::IO(i.to_string()))?;
            }
            let mut file =
                std::fs::File::create_new(&file_name).map_err(|i| AppError::IO(i.to_string()))?;
            file.write_all(include_bytes!("./config.toml"))
                .map_err(|i| AppError::IO(i.to_string()))?;
            file.flush().map_err(|i| AppError::IO(i.to_string()))?;
        }
        Ok(())
    }

    /// parse a given &str (read from the configfile) into Self
    fn parse(file_type: ConfigFileType, input: &str) -> Result<Self, AppError> {
        match file_type {
            ConfigFileType::Json => {
                serde_json::from_str::<Self>(input).map_err(|i| AppError::Parse(i.to_string()))
            }
            ConfigFileType::Jsonc | ConfigFileType::JsoncAsJson => {
                serde_jsonc::from_str::<Self>(input).map_err(|i| AppError::Parse(i.to_string()))
            }
            ConfigFileType::Toml => {
                toml::from_str::<Self>(input).map_err(|i| AppError::Parse(i.message().to_owned()))
            }
        }
    }

    /// Read the config file path to string, then attempt to parse
    fn parse_config_file(file_type: ConfigFileType, path: &PathBuf) -> Result<Self, AppError> {
        let mut file = std::fs::File::open(path).map_err(|_| {
            AppError::IO(
                path.to_str()
                    .map_or_else(String::new, std::borrow::ToOwned::to_owned),
            )
        })?;
        let mut input = String::new();
        file.read_to_string(&mut input)
            .map_err(|i| AppError::IO(i.to_string()))?;
        Self::parse(file_type, &input)
    }

    /// Resolve conflict in the args, this is handled automatically by Clap, basically just by rejecting it
    /// But here we can just change the options - although maybe should be also reject to follow the same behaviour as Clap?
    fn resolve_conflict(&mut self) {
        if let Some(color) = self.color_logs.as_ref() {
            if *color {
                self.raw_logs = Some(false);
            }
        }
        if let Some(interval) = self.docker_interval.as_ref() {
            if interval < &1000 {
                self.docker_interval = Some(1000);
            }
        }
    }

    /// Try to parse the config file when the path is user supplied via cliargs
    pub fn try_parse_from_file(path: &str) -> Option<Self> {
        let path = PathBuf::from(path);
        let Ok(file_type) = ConfigFileType::try_from(&path) else {
            return None;
        };

        Self::parse_config_file(file_type, &path).map_or(None, |mut config_file| {
            config_file.resolve_conflict();
            Some(config_file)
        })
    }

    /// Parse a config file using default config_file location
    /// This is executed first, then the CLI args are read, and if they contain a "--config-file" entry, then Self::try_parse_from_file() is executed
    pub fn try_parse(in_container: bool) -> Option<Self> {
        let mut config = None;
        for file_type in [
            ConfigFileType::Toml,
            ConfigFileType::Jsonc,
            ConfigFileType::JsoncAsJson,
            ConfigFileType::Json,
        ] {
            if let Ok(mut config_file) =
                Self::parse_config_file(file_type, &file_type.get_default_filename(in_container))
            {
                Self::resolve_conflict(&mut config_file);

                config = Some(config_file);
                break;
            }
        }

        if config.is_none() {
            Self::create_example_file(in_container).ok();
        }

        config
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use crate::config::{AppColors, Keymap};

    use super::ConfigFile;

    #[test]
    /// ./config.toml parses fine - as this is used to write a file on disk, it's vital that this is always valid
    fn test_parse_config_toml_valid() {
        let example_toml = include_str!("./config.toml");
        let result = ConfigFile::parse(super::ConfigFileType::Toml, example_toml);
        assert!(result.is_ok());
    }

    #[test]
    /// make sure config.toml matches the default keymap
    fn test_parse_config_keymap_toml() {
        let example_toml = include_str!("./config.toml");
        let result = ConfigFile::parse(super::ConfigFileType::Toml, example_toml).unwrap();
        assert!(result.keymap.is_some());
        assert_eq!(Keymap::from(result.keymap), Keymap::new());
    }

    #[test]
    /// make sure example.config.jsonc matches the default keymap
    fn test_parse_config_keymap_jsonc() {
        let example_jsonc = include_str!("../../example_config/example.config.jsonc");
        let result = ConfigFile::parse(super::ConfigFileType::Jsonc, example_jsonc).unwrap();
        assert!(result.keymap.is_some());
        assert_eq!(Keymap::from(result.keymap), Keymap::new());
    }

    #[test]
    /// make sure example.config.json matches the default keymap
    fn test_parse_config_keymap_json() {
        let example_json = include_str!("../../example_config/example.config.json");
        let result = ConfigFile::parse(super::ConfigFileType::Json, example_json).unwrap();
        assert!(result.keymap.is_some());
        assert_eq!(Keymap::from(result.keymap), Keymap::new());
    }

    #[test]
    /// All configs parsed and are equal
    fn test_parse_config_keymap_all() {
        let example_jsonc = include_str!("../../example_config/example.config.jsonc");
        let result_jsonc = ConfigFile::parse(super::ConfigFileType::Jsonc, example_jsonc).unwrap();
        assert!(result_jsonc.keymap.is_some());
        let result_jsonc = result_jsonc.keymap.unwrap();

        let example_json = include_str!("../../example_config/example.config.json");
        let result_json = ConfigFile::parse(super::ConfigFileType::Json, example_json).unwrap();
        assert!(result_json.keymap.is_some());
        let result_json = result_json.keymap.unwrap();

        let example_toml = include_str!("./config.toml");
        let result_toml = ConfigFile::parse(super::ConfigFileType::Toml, example_toml).unwrap();
        assert!(result_toml.keymap.is_some());
        let result_toml = result_toml.keymap.unwrap();

        assert_eq!(Keymap::from(Some(result_toml.clone())), Keymap::new());
        assert_eq!(result_toml, result_jsonc);
        assert_eq!(result_jsonc, result_json);
    }

    #[test]
    /// make sure config.toml matches the default app colors
    fn test_parse_config_colors_toml() {
        let example_toml = include_str!("./config.toml");
        let result = ConfigFile::parse(super::ConfigFileType::Toml, example_toml).unwrap();
        assert!(result.colors.is_some());
        assert_eq!(AppColors::from(result.colors), AppColors::new());
    }

    #[test]
    /// make sure config.toml matches the default app colors
    fn test_parse_config_colors_jsonc() {
        let example_jsonc = include_str!("../../example_config/example.config.jsonc");
        let result = ConfigFile::parse(super::ConfigFileType::Jsonc, example_jsonc).unwrap();
        assert!(result.colors.is_some());
        assert_eq!(AppColors::from(result.colors), AppColors::new());
    }

    #[test]
    /// make sure config.toml matches the default app colors
    fn test_parse_config_colors_json() {
        let example_json = include_str!("../../example_config/example.config.json");
        let result = ConfigFile::parse(super::ConfigFileType::Json, example_json).unwrap();
        assert!(result.colors.is_some());
        assert_eq!(AppColors::from(result.colors), AppColors::new());
    }

    #[test]
    /// All configs parsed and are equal
    fn test_parse_config_colors_all() {
        let example_jsonc = include_str!("../../example_config/example.config.jsonc");
        let result_jsonc = ConfigFile::parse(super::ConfigFileType::Jsonc, example_jsonc).unwrap();
        assert!(result_jsonc.colors.is_some());
        let result_jsonc = result_jsonc.colors.unwrap();

        let example_json = include_str!("../../example_config/example.config.json");
        let result_json = ConfigFile::parse(super::ConfigFileType::Json, example_json).unwrap();
        assert!(result_json.colors.is_some());
        let result_json = result_json.colors.unwrap();

        let example_toml = include_str!("./config.toml");
        let result_toml = ConfigFile::parse(super::ConfigFileType::Toml, example_toml).unwrap();
        assert!(result_toml.colors.is_some());
        let result_toml = result_toml.colors.unwrap();

        assert_eq!(AppColors::from(Some(result_toml.clone())), AppColors::new());
        assert_eq!(result_toml, result_jsonc);
        assert_eq!(result_jsonc, result_json);
    }
}
