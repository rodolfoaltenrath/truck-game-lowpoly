//! Preferências e pose locais; arquivo limitado, sem dependência do diretório de instalação.
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub const MAX_SETTINGS: u64 = 4096;
#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub version: u8,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub paint: u8,
    pub shadows: bool,
    pub camera: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            position: [60.0, 1.6, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            paint: 0,
            shadows: false,
            camera: 0,
        }
    }
}
#[derive(Debug)]
pub enum SettingsError {
    Io(std::io::Error),
    TooLarge,
    Invalid,
}
impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            e => write!(f, "{e:?}"),
        }
    }
}
impl std::error::Error for SettingsError {}
impl From<std::io::Error> for SettingsError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), SettingsError> {
        if self.version != 1 || self.paint > 5 || self.camera > 2 {
            return Err(SettingsError::Invalid);
        }
        truck_sim::Pose {
            position: self.position,
            rotation: self.rotation,
        }
        .validate()
        .map_err(|_| SettingsError::Invalid)
    }
    pub fn load(path: &Path) -> Result<Self, SettingsError> {
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e.into()),
        };
        let mut bytes = Vec::with_capacity(MAX_SETTINGS as usize + 1);
        file.take(MAX_SETTINGS + 1).read_to_end(&mut bytes)?;
        if bytes.len() > MAX_SETTINGS as usize {
            return Err(SettingsError::TooLarge);
        }
        let value: Self = serde_json::from_slice(&bytes).map_err(|_| SettingsError::Invalid)?;
        value.validate()?;
        Ok(value)
    }
    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|_| SettingsError::Invalid)?;
        let temporary = path.with_extension("tmp");
        let mut file = std::fs::File::create(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(temporary, path)?;
        Ok(())
    }
}
pub fn data_dir() -> anyhow::Result<PathBuf> {
    directories::ProjectDirs::from("br", "TruckGame", "SulDoBrasil")
        .map(|p| p.data_local_dir().to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("diretório de dados indisponível; use --data-dir"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_wrong_version_bounds_and_rotation() {
        let mut s = Settings {
            version: 9,
            ..Default::default()
        };
        assert!(s.validate().is_err());
        s.version = 1;
        s.position[0] = f32::NAN;
        assert!(s.validate().is_err());
        s.position = [0.0; 3];
        s.rotation = [0.0; 4];
        assert!(s.validate().is_err());
    }
    #[test]
    fn files_are_bounded_and_roundtrip() {
        let path = std::env::temp_dir().join(format!("truck-settings-{}.json", std::process::id()));
        Settings::default().save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap().version, 1);
        std::fs::write(&path, vec![b' '; MAX_SETTINGS as usize + 1]).unwrap();
        assert!(matches!(
            Settings::load(&path),
            Err(SettingsError::TooLarge)
        ));
        std::fs::remove_file(path).unwrap();
    }
}
