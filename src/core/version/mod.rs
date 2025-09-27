use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{ContainerError, ContainerResult};

/// Semantic version for containers following semver format (major.minor.patch)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Creates a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Checks if this version is compatible with another version
    /// Compatible means same major version and this version >= other
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        self.major == other.major && self >= other
    }

    pub fn validate(&self) -> ContainerResult<()> {
        // All versions are valid in basic implementation
        // Can add more complex validation rules here
        Ok(())
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = ContainerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        
        if parts.len() != 3 {
            return Err(ContainerError::InvalidVersion {
                version: s.to_string(),
            });
        }

        let major = parts[0].parse::<u32>().map_err(|_| {
            ContainerError::InvalidVersion {
                version: s.to_string(),
            }
        })?;

        let minor = parts[1].parse::<u32>().map_err(|_| {
            ContainerError::InvalidVersion {
                version: s.to_string(),
            }
        })?;

        let patch = parts[2].parse::<u32>().map_err(|_| {
            ContainerError::InvalidVersion {
                version: s.to_string(),
            }
        })?;

        let version = Version::new(major, minor, patch);
        version.validate()?;
        Ok(version)
    }
}

#[cfg(test)]
mod tests;

