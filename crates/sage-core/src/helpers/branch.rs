use std::{borrow::Borrow, fmt, ops::Deref, str::FromStr};

/// Wrapper around a branch name.
#[derive(Clone, Eq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct BranchName(String);

impl BranchName {
    pub fn new<S: Into<String>>(s: S) -> Result<Self, BranchNameError> {
        let s = s.into();
        if !is_valid(&s) {
            return Err(BranchNameError::Invalid(s.to_owned()));
        }
        Ok(Self(s.to_owned()))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for BranchName {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for BranchName {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for BranchName {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for BranchName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<BranchName> for String {
    #[inline]
    fn from(b: BranchName) -> Self {
        b.into_inner()
    }
}

impl<'a> TryFrom<&'a str> for BranchName {
    type Error = BranchNameError;
    #[inline]
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for BranchName {
    type Err = BranchNameError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_owned())
    }
}

impl PartialEq for BranchName {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<String> for BranchName {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<BranchName> for String {
    fn eq(&self, other: &BranchName) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&str> for BranchName {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<BranchName> for &str {
    fn eq(&self, other: &BranchName) -> bool {
        *self == other.as_str()
    }
}

mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for BranchName {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for BranchName {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            BranchName::new(s).map_err(serde::de::Error::custom)
        }
    }
}

/// Error returned when constructing a [`BranchName`]
#[derive(Debug, thiserror::Error)]
pub enum BranchNameError {
    #[error("invalid branch name: `{0}`")]
    Invalid(String),
}

fn is_valid(name: &str) -> bool {
    if name.is_empty() || name.ends_with('/') || name.starts_with('-') {
        return false;
    }
    if name.contains([' ', '~', '^', ':', '?', '*', '[', '\\'])
        || name.contains("//")
        || name.contains("..")
        || name.ends_with(".lock")
    {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let branch = BranchName::new("feature/cool").unwrap();
        assert_eq!(branch.as_str(), "feature/cool");
        let s: String = branch.clone().into();
        assert_eq!(s, "feature/cool");
        let borrowed: &str = branch.borrow();
        assert_eq!(borrowed, "feature/cool");
    }
}
