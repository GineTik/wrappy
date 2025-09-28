use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use regex::Regex;

use crate::shared::error::{ContainerError, ContainerResult};

/// Semantic version for containers following semver format (major.minor.patch)
/// Stored as string to preserve exact format and enable flexible validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Version {
    version: String,
}

impl Version {
    /// Creates a new version from string format
    pub fn new(version: &str) -> ContainerResult<Self> {
        let instance = Self {
            version: version.to_string(),
        };
        instance.validate()?;
        Ok(instance)
    }

    /// Creates version from individual components
    pub fn from_parts(major: u32, minor: u32, patch: u32) -> ContainerResult<Self> {
        let version_string = format!("{}.{}.{}", major, minor, patch);
        Self::new(&version_string)
    }

    /// Validates version format using semver specification
    pub fn validate(&self) -> ContainerResult<()> {
        Self::validate_version_format(&self.version)
    }

    /// Validates version string format
    fn validate_version_format(version: &str) -> ContainerResult<()> {
        let semver_regex = Regex::new(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")
            .map_err(|_| ContainerError::InvalidVersion {
                version: version.to_string(),
            })?;

        if !semver_regex.is_match(version) {
            return Err(ContainerError::InvalidVersion {
                version: version.to_string(),
            });
        }

        Ok(())
    }

    /// Parses version string into components
    fn parse_components(&self) -> ContainerResult<(u32, u32, u32)> {
        let parts: Vec<&str> = self.version.split('.').collect();
        
        if parts.len() != 3 {
            return Err(ContainerError::InvalidVersion {
                version: self.version.clone(),
            });
        }

        let major = parts[0].parse::<u32>().map_err(|_| {
            ContainerError::InvalidVersion {
                version: self.version.clone(),
            }
        })?;

        let minor = parts[1].parse::<u32>().map_err(|_| {
            ContainerError::InvalidVersion {
                version: self.version.clone(),
            }
        })?;

        let patch = parts[2].parse::<u32>().map_err(|_| {
            ContainerError::InvalidVersion {
                version: self.version.clone(),
            }
        })?;

        Ok((major, minor, patch))
    }

    /// Checks if this version is compatible with another version
    /// Compatible means same major version and this version >= other
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        match (self.parse_components(), other.parse_components()) {
            (Ok((s_major, s_minor, s_patch)), Ok((o_major, o_minor, o_patch))) => {
                s_major == o_major && (s_major, s_minor, s_patch) >= (o_major, o_minor, o_patch)
            }
            _ => false,
        }
    }

    /// Returns version as string
    pub fn as_str(&self) -> &str {
        &self.version
    }

    /// Gets major version number
    pub fn major(&self) -> ContainerResult<u32> {
        let (major, _, _) = self.parse_components()?;
        Ok(major)
    }

    /// Gets minor version number
    pub fn minor(&self) -> ContainerResult<u32> {
        let (_, minor, _) = self.parse_components()?;
        Ok(minor)
    }

    /// Gets patch version number
    pub fn patch(&self) -> ContainerResult<u32> {
        let (_, _, patch) = self.parse_components()?;
        Ok(patch)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)
    }
}

impl FromStr for Version {
    type Err = ContainerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::new(s)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.parse_components(), other.parse_components()) {
            (Ok((s_major, s_minor, s_patch)), Ok((o_major, o_minor, o_patch))) => {
                (s_major, s_minor, s_patch).cmp(&(o_major, o_minor, o_patch))
            }
            (Ok(_), Err(_)) => std::cmp::Ordering::Greater,
            (Err(_), Ok(_)) => std::cmp::Ordering::Less,
            (Err(_), Err(_)) => std::cmp::Ordering::Equal,
        }
    }
}


