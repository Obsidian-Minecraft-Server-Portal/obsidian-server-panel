use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ModVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
}

impl Display for ModVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for ModVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        // parse out build versions denoted by +, ex: 1.2.3+789
        let parts: Vec<&str> = parts[0..3].iter().map(|&p| p.split('+').next().unwrap()).collect();
        if parts.len() != 3 {
            return Err(format!("Version must be in the format 'major.minor.patch', '{}' was provided", s));
        }

        let major = parts[0].parse::<u32>().map_err(|_| "Invalid major version".to_string())?;
        let minor = parts[1].parse::<u32>().map_err(|_| "Invalid minor version".to_string())?;
        let patch = parts[2].parse::<u32>().map_err(|_| "Invalid patch version".to_string())?;

        Ok(ModVersion::new(major, minor, patch))
    }
}
