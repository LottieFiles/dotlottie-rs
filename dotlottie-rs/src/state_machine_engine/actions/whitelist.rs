use glob::Pattern;
use std::collections::HashSet;
use url::{ParseError, Url};

#[derive(Debug)]
pub struct Whitelist {
    entries: HashSet<WhitelistEntry>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum WhitelistEntry {
    Exact(String),
    Wildcard(String, Pattern),
}

impl Whitelist {
    pub fn new() -> Self {
        Whitelist {
            entries: HashSet::new(),
        }
    }

    fn normalize_url(url: &str) -> Result<String, String> {
        let url = url.trim().to_lowercase();
        let parsed_url = match Url::parse(&url) {
            Ok(url) => url,
            Err(ParseError::RelativeUrlWithoutBase) => Url::parse(&format!("https://{}", url))
                .map_err(|e| format!("Invalid URL: {}", e))?,
            Err(e) => return Err(format!("Invalid URL: {}", e)),
        };

        let host = parsed_url.host_str().ok_or("URL has no host")?;
        let path = parsed_url.path();

        // Normalize path: remove trailing slash unless it's just "/"
        let path = if path == "/" {
            ""
        } else {
            path.trim_end_matches('/')
        };
        Ok(format!("{}{}", host, path))
    }

    pub fn add(&mut self, pattern: &str) -> Result<(), String> {
        let pattern = pattern.trim().to_lowercase();

        if pattern.contains('*') {
            let base_domain = pattern
                .split('*')
                .next()
                .ok_or("Invalid wildcard pattern")?
                .trim_end_matches('/');

            // Create a more permissive glob pattern
            let mut glob_pattern = pattern.to_string();
            if !glob_pattern.ends_with('*') {
                glob_pattern.push('*');
            }

            let glob_pattern =
                Pattern::new(&glob_pattern).map_err(|e| format!("Invalid glob pattern: {}", e))?;

            self.entries.insert(WhitelistEntry::Wildcard(
                base_domain.to_string(),
                glob_pattern,
            ));
        } else {
            let normalized_pattern = Self::normalize_url(&pattern)?;
            self.entries
                .insert(WhitelistEntry::Exact(normalized_pattern));
        }

        Ok(())
    }

    pub fn is_allowed(&self, url_str: &str) -> Result<bool, String> {
        let normalized_url = Self::normalize_url(url_str)?;

        for entry in &self.entries {
            match entry {
                WhitelistEntry::Exact(pattern) => {
                    if normalized_url == *pattern {
                        return Ok(true);
                    }
                }
                WhitelistEntry::Wildcard(base_domain, pattern) => {
                    if normalized_url.starts_with(base_domain)
                        && (pattern.matches(&normalized_url)
                            || pattern.matches(&format!("{}/", normalized_url)))
                    {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitelist() {
        let mut whitelist = Whitelist::new();

        // Add some patterns
        whitelist.add("example.com/*").unwrap();
        whitelist.add("https://test.com/specific/path").unwrap();
        whitelist.add("api.domain.com/v1/*").unwrap();

        // Test exact matches
        assert!(whitelist
            .is_allowed("https://test.com/specific/path")
            .unwrap());
        assert!(whitelist.is_allowed("test.com/specific/path").unwrap());

        // Test wildcard matches
        assert!(whitelist.is_allowed("example.com/anything").unwrap());
        assert!(whitelist
            .is_allowed("example.com/path/to/resource")
            .unwrap());
        assert!(whitelist.is_allowed("api.domain.com/v1/users").unwrap());

        // Test non-matches
        assert!(!whitelist.is_allowed("other.com/path").unwrap());
        assert!(!whitelist.is_allowed("test.com/wrong/path").unwrap());
        assert!(!whitelist.is_allowed("api.domain.com/v2/users").unwrap());
    }

    #[test]
    fn test_google_casewhitelist() {
        let mut whitelist = Whitelist::new();

        // Add some patterns
        whitelist.add("www.google.com/*").unwrap();

        // Test root path with and without trailing slash
        assert!(whitelist.is_allowed("https://www.google.com/").unwrap());
        assert!(whitelist.is_allowed("https://www.google.com").unwrap());
        assert!(whitelist.is_allowed("www.google.com/").unwrap());
        assert!(whitelist.is_allowed("www.google.com").unwrap());

        // Test subpaths
        assert!(whitelist.is_allowed("www.google.com/search").unwrap());
        assert!(whitelist.is_allowed("www.google.com/search/").unwrap());

        // Test non-matches
        assert!(!whitelist.is_allowed("other.com/path").unwrap());
        assert!(!whitelist.is_allowed("api.google.com").unwrap());
    }
}
