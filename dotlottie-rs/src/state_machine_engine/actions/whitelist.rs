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
        // Split pattern into domain and path parts
        let (pattern_domain, pattern_path) = if let Some(slash_pos) = pattern.find('/') {
            (&pattern[..slash_pos], &pattern[slash_pos..])
        } else {
            (pattern, "")
        };

        // Split text into domain and path parts
        let (text_domain, text_path) = if let Some(slash_pos) = text.find('/') {
            (&text[..slash_pos], &text[slash_pos..])
        } else {
            (text, "")
        };

        // Check if domain matches (including wildcards)
        if !Self::matches_domain_wildcard(pattern_domain, text_domain) {
            return false;
        }

        // If pattern has no path component:
        // - If pattern ends with "/" explicitly, match root only
        // - If pattern ends with "/*", match any path
        // - Otherwise (just domain), match root only for backward compatibility
        if pattern_path.is_empty() {
            // Domain-only pattern - match only root (no path)
            return text_path.is_empty();
        }

        // If pattern has a path but text doesn't, it's a mismatch
        // Exception: if pattern path is just "/" or "/*", it should match URLs with no path
        if text_path.is_empty() && pattern_path != "/" && pattern_path != "/*" {
            return false;
        }

        // Check if path matches (including wildcards)
        Self::matches_path_wildcard(pattern_path, text_path)
    }

    fn matches_domain_wildcard(pattern: &str, text: &str) -> bool {
        if !pattern.contains('*') {
            return pattern == text;
        }

        // Special case: if pattern starts with "*." and has no other wildcards after that,
        // it matches any number of subdomains
        if let Some(base_domain) = pattern.strip_prefix("*.") {
            // Check if there are more wildcards in the base domain
            if !base_domain.contains('*') {
                // This is a simple prefix wildcard like "*.example.com"
                // Check if text ends with the base domain
                if text == base_domain {
                    // Exact match without subdomains
                    return true;
                }

                // Check if text ends with ".base_domain"
                let suffix = format!(".{base_domain}");
                if text.ends_with(&suffix) {
                    return true;
                }

                return false;
            }
        }

        // Split domains into parts
        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let text_parts: Vec<&str> = text.split('.').collect();

        // Domains must have the same number of parts for position-specific wildcards
        if pattern_parts.len() != text_parts.len() {
            return false;
        }

        // Check each part
        for (pattern_part, text_part) in pattern_parts.iter().zip(text_parts.iter()) {
            if *pattern_part == "*" {
                // Wildcard matches any part
                continue;
            } else if *pattern_part != *text_part {
                // Non-wildcard parts must match exactly
                return false;
            }
        }

        true
    }

    fn matches_path_wildcard(pattern: &str, text: &str) -> bool {
        if !pattern.contains('*') {
            return pattern == text;
        }

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
            "/"
        } else {
            path.trim_end_matches('/')
        };
        Ok(format!("{host}{path}"))
    }

    pub fn add(&mut self, pattern: &str) -> Result<(), String> {
        let pattern = pattern.trim().to_lowercase();

        if pattern.contains('*') {
            let normalized_pattern = if let Some(without_protocol) = pattern
                .strip_prefix("https://")
                .or_else(|| pattern.strip_prefix("http://"))
            {
                // For URLs with protocol, parse and normalize while preserving wildcards
                if let Some(slash_pos) = without_protocol.find('/') {
                    let host = &without_protocol[..slash_pos];
                    let path = &without_protocol[slash_pos..];
                    // For patterns, preserve the slash if it's just "/" to distinguish from no path
                    let path = if path == "/" {
                        "/"
                    } else {
                        path.trim_end_matches('/')
                    };
                    format!("{host}{path}")
                } else {
                    without_protocol.to_string()
                }
            } else {
                // For patterns without protocol, normalize path if present
                if let Some(slash_pos) = pattern.find('/') {
                    let host = &pattern[..slash_pos];
                    let path = &pattern[slash_pos..];
                    // For patterns, preserve the slash if it's just "/" to distinguish from no path
                    let path = if path == "/" {
                        "/"
                    } else {
                        path.trim_end_matches('/')
                    };
                    format!("{host}{path}")
                } else {
                    pattern.clone()
                }
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

        // Special case: if "*" is in the whitelist, allow everything
        for entry in &self.entries {
            match entry {
                WhitelistEntry::Exact(pattern) => {
                    if pattern == "*" {
                        return Ok(true);
                    }
                }
                WhitelistEntry::Wildcard(pattern) => {
                    if pattern == "*" {
                        return Ok(true);
                    }
                }
            }
        }

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

    #[test]
    fn test_domain_wildcards() {
        let mut whitelist = Whitelist::new();

        // Add domain wildcard pattern
        whitelist.add("www.*.google.com/*").unwrap();

        // Test matching domains
        assert!(whitelist.is_allowed("www.test.google.com/test").unwrap());
        assert!(whitelist.is_allowed("www.dev.google.com/api").unwrap());
        assert!(whitelist.is_allowed("www.staging.google.com/").unwrap());
        assert!(whitelist.is_allowed("www.prod.google.com").unwrap());

        // Test non-matching domains
        assert!(!whitelist.is_allowed("www.google.com/test").unwrap());
        assert!(!whitelist.is_allowed("api.test.google.com/test").unwrap());
        assert!(!whitelist.is_allowed("www.test.google.org/test").unwrap());
        // Note: www.test.google.com should match because /* allows any path including no path

        // Test with protocol
        assert!(whitelist
            .is_allowed("https://www.test.google.com/test")
            .unwrap());
        assert!(whitelist
            .is_allowed("http://www.dev.google.com/api")
            .unwrap());
    }

    #[test]
    fn test_multiple_domain_wildcards() {
        let mut whitelist = Whitelist::new();

        // Add pattern with multiple wildcards
        whitelist.add("*.test.*.google.com/*").unwrap();

        // Test matching domains
        assert!(whitelist
            .is_allowed("www.test.dev.google.com/test")
            .unwrap());
        assert!(whitelist
            .is_allowed("api.test.staging.google.com/api")
            .unwrap());

        // Test non-matching domains
        assert!(!whitelist.is_allowed("www.test.google.com/test").unwrap()); // Missing subdomain
        assert!(!whitelist
            .is_allowed("www.dev.test.google.com/test")
            .unwrap()); // Wrong order
    }

    #[test]
    fn test_domain_wildcard_without_path() {
        let mut whitelist = Whitelist::new();

        // Add domain wildcard pattern without path
        whitelist.add("www.*.google.com").unwrap();

        // Test matching domains (any path should work)
        assert!(whitelist.is_allowed("www.staging.google.com").unwrap());
        
        // Test non-matching domains
        assert!(!whitelist.is_allowed("www.dev.google.com/").unwrap());
        assert!(!whitelist.is_allowed("www.test.google.com/test").unwrap());
        assert!(!whitelist.is_allowed("www.google.com/test").unwrap());
        assert!(!whitelist.is_allowed("api.test.google.com/test").unwrap());
    }

    #[test]
    fn test_wildcard_allows_everything() {
        let mut whitelist = Whitelist::new();

        // Add wildcard pattern that allows everything
        whitelist.add("*").unwrap();

        // Test that any URL is allowed
        assert!(whitelist.is_allowed("https://example.com").unwrap());
        assert!(whitelist.is_allowed("http://test.com/path").unwrap());
        assert!(whitelist.is_allowed("www.google.com/search").unwrap());
        assert!(whitelist.is_allowed("api.domain.com/v1/users").unwrap());
        assert!(whitelist.is_allowed("localhost:3000").unwrap());
        assert!(whitelist.is_allowed("192.168.1.1").unwrap());
    }

    #[test]
    fn test_wildcard_with_other_patterns() {
        let mut whitelist = Whitelist::new();

        // Add both wildcard and specific patterns
        whitelist.add("*").unwrap();
        whitelist.add("example.com/*").unwrap();
        whitelist.add("https://test.com/specific/path").unwrap();

        // Wildcard should take precedence and allow everything
        assert!(whitelist.is_allowed("https://example.com").unwrap());
        assert!(whitelist.is_allowed("http://test.com/path").unwrap());
        assert!(whitelist.is_allowed("www.google.com/search").unwrap());
        assert!(whitelist.is_allowed("api.domain.com/v1/users").unwrap());
        assert!(whitelist.is_allowed("localhost:3000").unwrap());
        assert!(whitelist.is_allowed("192.168.1.1").unwrap());
    }

    #[test]
    fn test_prefix_wildcard() {
        let mut whitelist = Whitelist::new();

        // Add prefix wildcard pattern
        whitelist.add("*.lottiefiles.com").unwrap();

        // Test matching with multiple subdomain levels (root only)
        assert!(whitelist.is_allowed("www.lottiefiles.com").unwrap());
        assert!(whitelist.is_allowed("editor.lottiefiles.com").unwrap());
        assert!(whitelist.is_allowed("www.editor.lottiefiles.com").unwrap());
        assert!(whitelist.is_allowed("www.creator.lottiefiles.com").unwrap());
        assert!(whitelist
            .is_allowed("api.v2.staging.lottiefiles.com")
            .unwrap());
        assert!(whitelist.is_allowed("a.b.c.d.lottiefiles.com").unwrap());

        // Test exact match (no subdomain)
        assert!(whitelist.is_allowed("lottiefiles.com").unwrap());

        // Test that paths are NOT allowed with domain-only pattern
        assert!(!whitelist
            .is_allowed("www.editor.lottiefiles.com/path")
            .unwrap());
        assert!(!whitelist.is_allowed("api.lottiefiles.com/v1/data").unwrap());
        assert!(!whitelist.is_allowed("www.lottiefiles.com/editor").unwrap());

        // Test non-matches
        assert!(!whitelist.is_allowed("lottiefiles.org").unwrap());
        assert!(!whitelist.is_allowed("notlottiefiles.com").unwrap());
        assert!(!whitelist.is_allowed("www.lottiefiles.com.fake").unwrap());
    }

    #[test]
    fn test_prefix_wildcard_with_path() {
        let mut whitelist = Whitelist::new();

        // Add prefix wildcard pattern with path wildcard
        whitelist.add("*.example.com/*").unwrap();

        // Test matching with various subdomain levels
        assert!(whitelist.is_allowed("www.example.com/test").unwrap());
        assert!(whitelist.is_allowed("api.example.com/v1").unwrap());
        assert!(whitelist
            .is_allowed("dev.staging.example.com/deploy")
            .unwrap());
        assert!(whitelist
            .is_allowed("a.b.c.example.com/path/to/resource")
            .unwrap());

        // Test root paths
        assert!(whitelist.is_allowed("www.example.com/").unwrap());
        assert!(whitelist.is_allowed("www.example.com").unwrap());
        assert!(whitelist.is_allowed("example.com").unwrap());

        // Test non-matches
        assert!(!whitelist.is_allowed("example.org/test").unwrap());
        assert!(!whitelist.is_allowed("notexample.com/test").unwrap());
    }

    #[test]
    fn test_prefix_wildcard_with_specific_path() {
        let mut whitelist = Whitelist::new();

        // Add prefix wildcard pattern with specific path
        whitelist.add("*.api.com/v1/*").unwrap();

        // Test matching
        assert!(whitelist.is_allowed("www.api.com/v1/users").unwrap());
        assert!(whitelist.is_allowed("staging.api.com/v1/data").unwrap());
        assert!(whitelist.is_allowed("dev.test.api.com/v1/info").unwrap());

        // Test non-matches (wrong path)
        assert!(!whitelist.is_allowed("www.api.com/v2/users").unwrap());
        assert!(!whitelist.is_allowed("staging.api.com/users").unwrap());
    }

    #[test]
    fn test_all_allowed() {
        let mut whitelist = Whitelist::new();

        // Add wildcard that allows everything
        whitelist.add("*").unwrap();

        // Everything should be allowed when "*" is in the whitelist
        assert!(whitelist
            .is_allowed("https://www.api.com/v1/users")
            .unwrap());
        assert!(whitelist.is_allowed("www.api.com/v1/users").unwrap());
        assert!(whitelist.is_allowed("staging.api.com/v1/data").unwrap());
        assert!(whitelist.is_allowed("dev.test.api.com/v1/info").unwrap());
        assert!(whitelist.is_allowed("www.api.com/v2/users").unwrap());
        assert!(whitelist.is_allowed("staging.api.com/users").unwrap());
        assert!(whitelist.is_allowed("anything.goes.here").unwrap());
        assert!(whitelist.is_allowed("192.168.1.1").unwrap());
        assert!(whitelist.is_allowed("localhost:3000").unwrap());
    }
}
