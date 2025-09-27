#[cfg(test)]
mod tests {
    use super::*;
    use crate::Version;

    #[test]
    fn test_version_creation() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_display() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_version_from_string() {
        let version: Version = "1.2.3".parse().unwrap();
        assert_eq!(version, Version::new(1, 2, 3));
    }

    #[test]
    fn test_invalid_version_format() {
        let result: Result<Version, _> = "1.2".parse();
        assert!(result.is_err());
        
        let result: Result<Version, _> = "1.2.3.4".parse();
        assert!(result.is_err());
        
        let result: Result<Version, _> = "1.a.3".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_version_compatibility() {
        let v1_2_3 = Version::new(1, 2, 3);
        let v1_2_4 = Version::new(1, 2, 4);
        let v1_3_0 = Version::new(1, 3, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        assert!(v1_2_4.is_compatible_with(&v1_2_3));
        assert!(v1_3_0.is_compatible_with(&v1_2_3));
        assert!(!v1_2_3.is_compatible_with(&v1_2_4));
        assert!(!v2_0_0.is_compatible_with(&v1_2_3));
        assert!(!v1_2_3.is_compatible_with(&v2_0_0));
    }

    #[test]
    fn test_version_ordering() {
        let v1_0_0 = Version::new(1, 0, 0);
        let v1_0_1 = Version::new(1, 0, 1);
        let v1_1_0 = Version::new(1, 1, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        assert!(v1_0_0 < v1_0_1);
        assert!(v1_0_1 < v1_1_0);
        assert!(v1_1_0 < v2_0_0);
    }
}