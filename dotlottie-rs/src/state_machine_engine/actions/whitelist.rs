use std::collections::HashSet;

#[derive(Debug)]
pub struct Whitelist {
    entries: HashSet<WhitelistEntry>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum WhitelistEntry {
    Exact(String),
    Wildcard(String),
}

fn parse_url(url_str: &str) -> Result<(String, String), String> {
    let url_str = url_str.trim();

    if url_str.is_empty() {
        return Err("Empty URL".to_string());
    }

    let without_protocol = if let Some(stripped) = url_str.strip_prefix("https://") {
        stripped
    } else if let Some(stripped) = url_str.strip_prefix("http://") {
        stripped
    } else {
        url_str
    };

    if without_protocol.is_empty() {
        return Err("URL has no host".to_string());
    }

    if let Some(slash_pos) = without_protocol.find('/') {
        let host = &without_protocol[..slash_pos];
        let path = &without_protocol[slash_pos..];

        if host.is_empty() {
            return Err("URL has no host".to_string());
        }

        Ok((host.to_string(), path.to_string()))
    } else {
        Ok((without_protocol.to_string(), String::new()))
    }
}

impl WhitelistEntry {
    fn matches_wildcard(pattern: &str, text: &str) -> bool {
        if let Some(star_pos) = pattern.find('*') {
            let prefix = &pattern[..star_pos];
            let suffix = &pattern[star_pos + 1..];

            let (prefix_matches, effective_prefix_len) =
                if prefix.ends_with('/') && !text.contains('/') {
                    let prefix_without_slash = &prefix[..prefix.len() - 1];
                    let matches = text.starts_with(prefix_without_slash)
                        && (text.len() == prefix_without_slash.len()
                            || text.starts_with(&format!("{prefix_without_slash}/")));
                    (matches, prefix_without_slash.len())
                } else {
                    let matches = text.starts_with(prefix);
                    (matches, prefix.len())
                };

            if !prefix_matches {
                return false;
            }

            if text.len() < effective_prefix_len + suffix.len() {
                return false;
            }

            suffix.is_empty() || text.ends_with(suffix)
        } else {
            pattern == text
        }
    }
}

impl Default for Whitelist {
    fn default() -> Self {
        Self::new()
    }
}

impl Whitelist {
    pub fn new() -> Self {
        Whitelist {
            entries: HashSet::new(),
        }
    }

    fn normalize_url(url: &str) -> Result<String, String> {
        let url = url.trim().to_lowercase();

        let (host, path) = match parse_url(&url) {
            Ok(result) => result,
            Err(_) => match parse_url(&format!("https://{url}")) {
                Ok(result) => result,
                Err(e) => return Err(format!("Invalid URL: {e}")),
            },
        };

        let path = if path == "/" {
            ""
        } else {
            path.trim_end_matches('/')
        };
        Ok(format!("{host}{path}"))
    }

    pub fn add(&mut self, pattern: &str) -> Result<(), String> {
        let pattern = pattern.trim().to_lowercase();

        if pattern.contains('*') {
            let normalized_pattern =
                if pattern.starts_with("http://") || pattern.starts_with("https://") {
                    match parse_url(&pattern.replace('*', "placeholder")) {
                        Ok((host, path)) => {
                            let path = path.trim_end_matches("placeholder");
                            let path = if path.is_empty() {
                                ""
                            } else {
                                path.trim_end_matches('/')
                            };
                            format!("{host}{path}*")
                        }
                        Err(_) => pattern.clone(),
                    }
                } else {
                    pattern.clone()
                };

            self.entries
                .insert(WhitelistEntry::Wildcard(normalized_pattern));
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
                WhitelistEntry::Wildcard(pattern) => {
                    if WhitelistEntry::matches_wildcard(pattern, &normalized_url) {
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
    fn test_custom_url_parsing() {
        assert_eq!(
            parse_url("example.com").unwrap(),
            ("example.com".to_string(), "".to_string())
        );
        assert_eq!(
            parse_url("example.com/path").unwrap(),
            ("example.com".to_string(), "/path".to_string())
        );

        assert_eq!(
            parse_url("https://example.com").unwrap(),
            ("example.com".to_string(), "".to_string())
        );
        assert_eq!(
            parse_url("http://example.com/path").unwrap(),
            ("example.com".to_string(), "/path".to_string())
        );
        assert_eq!(
            parse_url("https://example.com/path/to/resource").unwrap(),
            ("example.com".to_string(), "/path/to/resource".to_string())
        );

        assert!(parse_url("").is_err());
        assert!(parse_url("https://").is_err());
        assert!(parse_url("http://").is_err());

        assert_eq!(
            parse_url("  example.com  ").unwrap(),
            ("example.com".to_string(), "".to_string())
        );
    }

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
