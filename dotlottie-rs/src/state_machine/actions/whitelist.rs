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

/// Split a URL into `(host, path)`, ignoring any scheme. `None` when there is
/// no host to speak of.
fn parse_url(url_str: &str) -> Option<(&str, &str)> {
    let url_str = url_str.trim();

    let without_protocol = url_str
        .strip_prefix("https://")
        .or_else(|| url_str.strip_prefix("http://"))
        .unwrap_or(url_str);

    if without_protocol.is_empty() {
        return None;
    }

    let host_len = without_protocol.find('/').unwrap_or(without_protocol.len());

    (host_len > 0).then(|| without_protocol.split_at(host_len))
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

        // Special case: if pattern path is "/*", it should match both root (empty path) and any path
        if pattern_path == "/*" {
            return true; // Match root and any path
        }

        // If pattern has a path but text doesn't, it's a mismatch
        // Exception: if pattern path is just "/"
        if text_path.is_empty() && pattern_path != "/" {
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

        // Special case: if pattern is "/*", it should match root ("/") and any path
        if pattern == "/*" {
            return true; // Match everything including empty/root path
        }

        // Check if pattern ends with /*/ (exactly one segment) vs /* (anything)
        let expects_single_segment = pattern.ends_with("/*/");

        // Handle multiple wildcards by splitting the pattern at wildcards
        let parts: Vec<&str> = pattern.split('*').collect();

        // If pattern is just "*", match everything
        if parts.len() == 1 && parts[0].is_empty() {
            return true;
        }

        let mut text_pos = 0;

        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                // Empty part means wildcard at beginning/end or consecutive wildcards
                if i == 0 {
                    // Leading wildcard - continue
                    continue;
                } else if i == parts.len() - 1 {
                    // Trailing wildcard
                    if expects_single_segment {
                        // Pattern ends with /*/ - match exactly one segment
                        // The previous part should have ended with /, and we need
                        // to check that there's exactly one segment after it
                        let remaining = &text[text_pos..];

                        // Count slashes in the remaining text
                        let slash_count = remaining.chars().filter(|c| *c == '/').count();

                        // For exactly one segment with trailing slash, we expect:
                        // - If remaining text has no slashes: it's one segment without trailing slash (invalid)
                        // - If remaining text has one slash at the end: it's one segment with trailing slash (valid)
                        // - If remaining text has more slashes: it's multiple segments (invalid)
                        return slash_count == 1 && remaining.ends_with('/');
                    } else {
                        // Pattern ends with /* - match rest of string
                        return true;
                    }
                } else {
                    // Consecutive wildcards or middle wildcard - continue
                    continue;
                }
            }

            // Special handling for "/" when it's the only non-wildcard part
            // This handles cases like "/*" where parts = ["/", ""]
            if *part == "/" && i == 0 && parts.len() == 2 && parts[1].is_empty() {
                // This is the "/*" pattern - should match everything
                return true;
            }

            // Find the next occurrence of this part in the remaining text
            if let Some(found_pos) = text[text_pos..].find(part) {
                let absolute_pos = text_pos + found_pos;

                // If this is the first part, it must be at the beginning (unless pattern starts with *)
                if i == 0 && found_pos != 0 {
                    return false;
                }

                // If this is the last part, it must be at the end (unless pattern ends with *)
                if i == parts.len() - 1 && absolute_pos + part.len() != text.len() {
                    return false;
                }

                // Move position past this part
                text_pos = absolute_pos + part.len();
            } else {
                // Part not found in remaining text
                return false;
            }
        }

        true
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

    /// Lowercase `host` + `path`, scheme and trailing slash stripped.
    fn normalize_url(url: &str) -> Option<String> {
        let url = url.trim().to_ascii_lowercase();
        let (host, path) = parse_url(&url)?;

        // Empty path and "/" both become empty (representing root)
        let path = if path == "/" {
            ""
        } else {
            path.trim_end_matches('/')
        };

        let mut normalized = String::with_capacity(host.len() + path.len());
        normalized.push_str(host);
        normalized.push_str(path);
        Some(normalized)
    }

    /// Add a pattern. A pattern with no parseable host is ignored.
    pub fn add(&mut self, pattern: &str) {
        let pattern = pattern.trim().to_ascii_lowercase();

        if pattern.contains('*') {
            let normalized_pattern = if let Some(without_protocol) = pattern
                .strip_prefix("https://")
                .or_else(|| pattern.strip_prefix("http://"))
            {
                // For URLs with protocol, parse and normalize while preserving wildcards
                if let Some(slash_pos) = without_protocol.find('/') {
                    let host = &without_protocol[..slash_pos];
                    let path = &without_protocol[slash_pos..];
                    // For wildcard patterns, preserve meaningful paths and trailing slashes
                    let path = if path == "/" {
                        "" // Root path becomes empty
                    } else if path == "/*" {
                        "/*" // Preserve /* as it has special meaning
                    } else if path.ends_with("/*/")
                        || (path.ends_with("/") && !path.ends_with("*/"))
                    {
                        path // Preserve trailing /*/ pattern as-is
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
                    // For wildcard patterns, preserve meaningful paths and trailing slashes
                    let path = if path == "/" {
                        "" // Root path becomes empty
                    } else if path == "/*" {
                        "/*" // Preserve /* as it has special meaning
                    } else if path.ends_with("/*/")
                        || (path.ends_with("/") && !path.ends_with("*/"))
                    {
                        path // Preserve trailing /*/ pattern as-is
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
            let Some(normalized_pattern) = Self::normalize_url(&pattern) else {
                return;
            };
            self.entries
                .insert(WhitelistEntry::Exact(normalized_pattern));
        }
    }

    /// An unparseable URL is never allowed.
    pub fn is_allowed(&self, url_str: &str) -> bool {
        let Some(normalized_url) = Self::normalize_url(url_str) else {
            return false;
        };

        self.entries.iter().any(|entry| match entry {
            WhitelistEntry::Exact(pattern) => pattern == "*" || *pattern == normalized_url,
            WhitelistEntry::Wildcard(pattern) => {
                pattern == "*" || WhitelistEntry::matches_wildcard(pattern, &normalized_url)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_url_parsing() {
        assert_eq!(parse_url("example.com"), Some(("example.com", "")));
        assert_eq!(
            parse_url("example.com/path"),
            Some(("example.com", "/path"))
        );
        assert_eq!(parse_url("https://example.com"), Some(("example.com", "")));
        assert_eq!(
            parse_url("http://example.com/path"),
            Some(("example.com", "/path"))
        );
        assert_eq!(
            parse_url("https://example.com/path/to/resource"),
            Some(("example.com", "/path/to/resource"))
        );

        assert_eq!(parse_url(""), None);
        assert_eq!(parse_url("https://"), None);
        assert_eq!(parse_url("http://"), None);

        assert_eq!(parse_url("  example.com  "), Some(("example.com", "")));
    }

    #[test]
    fn test_whitelist() {
        let mut whitelist = Whitelist::new();

        // Add some patterns
        whitelist.add("example.com/*");
        whitelist.add("https://test.com/specific/path");
        whitelist.add("api.domain.com/v1/*");

        // Test exact matches
        assert!(whitelist.is_allowed("https://test.com/specific/path"));
        assert!(whitelist.is_allowed("test.com/specific/path"));

        // Test wildcard matches
        assert!(whitelist.is_allowed("example.com/anything"));
        assert!(whitelist.is_allowed("example.com/path/to/resource"));
        assert!(whitelist.is_allowed("api.domain.com/v1/users"));

        // Test non-matches
        assert!(!whitelist.is_allowed("other.com/path"));
        assert!(!whitelist.is_allowed("test.com/wrong/path"));
        assert!(!whitelist.is_allowed("api.domain.com/v2/users"));
    }

    #[test]
    fn test_trailing_wildcard() {
        let mut whitelist = Whitelist::new();

        // Add some patterns
        whitelist.add("www.google.com/*");

        // Test root path with and without trailing slash
        assert!(whitelist.is_allowed("https://www.google.com/"));
        assert!(whitelist.is_allowed("https://www.google.com"));
        assert!(whitelist.is_allowed("www.google.com/"));
        assert!(whitelist.is_allowed("www.google.com"));

        // Test subpaths
        assert!(whitelist.is_allowed("www.google.com/search"));
        assert!(whitelist.is_allowed("www.google.com/search/"));

        // Test non-matches
        assert!(!whitelist.is_allowed("other.com/path"));
        assert!(!whitelist.is_allowed("api.google.com"));
    }

    #[test]
    fn test_wildcard_path_includes_root() {
        let mut whitelist = Whitelist::new();

        // Test that example.com/* allows both root and any subpath
        whitelist.add("example.com/*");

        // Should allow root
        assert!(whitelist.is_allowed("example.com"));
        assert!(whitelist.is_allowed("example.com/"));
        assert!(whitelist.is_allowed("https://example.com"));
        assert!(whitelist.is_allowed("https://example.com/"));
        assert!(whitelist.is_allowed("http://example.com"));
        assert!(whitelist.is_allowed("http://example.com/"));

        // Should also allow any subpath
        assert!(whitelist.is_allowed("example.com/path"));
        assert!(whitelist.is_allowed("example.com/path/to/resource"));
        assert!(whitelist.is_allowed("https://example.com/anything"));

        // Should not allow different domains
        assert!(!whitelist.is_allowed("other.com"));
        assert!(!whitelist.is_allowed("other.com/path"));
    }

    #[test]
    fn test_domain_wildcards() {
        let mut whitelist = Whitelist::new();

        // Add domain wildcard pattern
        whitelist.add("www.*.google.com/*");

        // Test matching domains
        assert!(whitelist.is_allowed("www.test.google.com/test"));
        assert!(whitelist.is_allowed("www.dev.google.com/api"));
        assert!(whitelist.is_allowed("www.staging.google.com/"));
        assert!(whitelist.is_allowed("www.prod.google.com"));

        // Test non-matching domains
        assert!(!whitelist.is_allowed("www.google.com/test"));
        assert!(!whitelist.is_allowed("api.test.google.com/test"));
        assert!(!whitelist.is_allowed("www.test.google.org/test"));
        // Note: www.test.google.com should match because /* allows any path including no path

        // Test with protocol
        assert!(whitelist.is_allowed("https://www.test.google.com/test"));
        assert!(whitelist.is_allowed("http://www.dev.google.com/api"));
    }

    #[test]
    fn test_multiple_domain_wildcards() {
        let mut whitelist = Whitelist::new();

        // Add pattern with multiple wildcards
        whitelist.add("*.test.*.google.com/*");

        // Test matching domains
        assert!(whitelist.is_allowed("www.test.dev.google.com/test"));
        assert!(whitelist.is_allowed("api.test.staging.google.com/api"));

        // Test non-matching domains
        assert!(!whitelist.is_allowed("www.test.google.com/test")); // Missing subdomain
        assert!(!whitelist.is_allowed("www.dev.test.google.com/test")); // Wrong order
    }

    #[test]
    fn test_domain_wildcard_without_path() {
        let mut whitelist = Whitelist::new();

        // Add domain wildcard pattern without path
        whitelist.add("www.*.google.com");

        // Test matching domains (any path should work)
        assert!(whitelist.is_allowed("www.staging.google.com"));
        assert!(whitelist.is_allowed("www.dev.google.com/"));

        // Test non-matching domains
        assert!(!whitelist.is_allowed("www.test.google.com/test"));
        assert!(!whitelist.is_allowed("www.google.com/test"));
        assert!(!whitelist.is_allowed("api.test.google.com/test"));
    }

    #[test]
    fn test_wildcard_with_other_patterns() {
        let mut whitelist = Whitelist::new();

        // Add both wildcard and specific patterns
        whitelist.add("*");
        whitelist.add("example.com/*");
        whitelist.add("https://test.com/specific/path");

        // Wildcard should take precedence and allow everything
        assert!(whitelist.is_allowed("https://example.com"));
        assert!(whitelist.is_allowed("http://test.com/path"));
        assert!(whitelist.is_allowed("www.google.com/search"));
        assert!(whitelist.is_allowed("api.domain.com/v1/users"));
        assert!(whitelist.is_allowed("localhost:3000"));
        assert!(whitelist.is_allowed("192.168.1.1"));
    }

    #[test]
    fn test_prefix_wildcard() {
        let mut whitelist = Whitelist::new();

        // Add prefix wildcard pattern
        whitelist.add("*.lottiefiles.com");

        // Test matching with multiple subdomain levels (root only)
        assert!(whitelist.is_allowed("www.lottiefiles.com"));
        assert!(whitelist.is_allowed("editor.lottiefiles.com"));
        assert!(whitelist.is_allowed("www.editor.lottiefiles.com"));
        assert!(whitelist.is_allowed("www.creator.lottiefiles.com"));
        assert!(whitelist.is_allowed("api.v2.staging.lottiefiles.com"));
        assert!(whitelist.is_allowed("a.b.c.d.lottiefiles.com"));
        assert!(whitelist.is_allowed("lottiefiles.com"));

        // Test that paths are NOT allowed with domain-only pattern
        assert!(!whitelist.is_allowed("www.editor.lottiefiles.com/path"));
        assert!(!whitelist.is_allowed("api.lottiefiles.com/v1/data"));
        assert!(!whitelist.is_allowed("www.lottiefiles.com/editor"));

        // Test non-matches
        assert!(!whitelist.is_allowed("lottiefiles.org"));
        assert!(!whitelist.is_allowed("notlottiefiles.com"));
        assert!(!whitelist.is_allowed("www.lottiefiles.com.fake"));
    }

    #[test]
    fn test_prefix_wildcard_with_specific_path() {
        let mut whitelist = Whitelist::new();

        whitelist.add("*.api.com/v1/*");

        assert!(whitelist.is_allowed("www.api.com/v1/users"));
        assert!(whitelist.is_allowed("staging.api.com/v1/data"));
        assert!(whitelist.is_allowed("dev.test.api.com/v1/info"));

        // Test non-matches (wrong path)
        assert!(!whitelist.is_allowed("www.api.com/v2/users"));
        assert!(!whitelist.is_allowed("staging.api.com/users"));
    }

    #[test]
    fn test_all_allowed() {
        let mut whitelist = Whitelist::new();

        whitelist.add("*");

        // Everything should be allowed when "*" is in the whitelist
        assert!(whitelist.is_allowed("https://www.api.com/v1/users"));
        assert!(whitelist.is_allowed("www.api.com/v1/users"));
        assert!(whitelist.is_allowed("staging.api.com/v1/data"));
        assert!(whitelist.is_allowed("dev.test.api.com/v1/info"));
        assert!(whitelist.is_allowed("www.api.com/v2/users"));
        assert!(whitelist.is_allowed("staging.api.com/users"));
        assert!(whitelist.is_allowed("anything.goes.here"));
        assert!(whitelist.is_allowed("192.168.1.1"));
        assert!(whitelist.is_allowed("localhost:3000"));
    }

    #[test]
    fn test_domain_only_pattern() {
        let mut whitelist = Whitelist::new();

        // Test that domain-only pattern (without /*) only allows root
        whitelist.add("example.com");

        // Should allow root only
        assert!(whitelist.is_allowed("example.com"));
        assert!(whitelist.is_allowed("https://example.com"));

        // Should NOT allow paths
        assert!(!whitelist.is_allowed("example.com/path"));
        assert!(!whitelist.is_allowed("example.com/anything"));
    }

    #[test]
    fn test_slash_vs_slash_star() {
        let mut whitelist = Whitelist::new();

        // Test the difference between "/" and "/*"
        whitelist.add("test.com/");
        assert!(whitelist.is_allowed("test.com/"));
        assert!(whitelist.is_allowed("test.com")); // Normalized to match

        assert!(!whitelist.is_allowed("test.com/path"));

        let mut whitelist2 = Whitelist::new();
        whitelist2.add("api.com/*");

        assert!(whitelist2.is_allowed("api.com"));
        assert!(whitelist2.is_allowed("api.com/"));
        assert!(whitelist2.is_allowed("api.com/v1"));
        assert!(whitelist2.is_allowed("api.com/v1/users"));
    }

    #[test]
    fn test_multiple_wildcards_in_path() {
        let mut whitelist = Whitelist::new();

        // Test the specific case with multiple wildcards in path
        whitelist.add("example.com/path/*/another/*");

        assert!(whitelist.is_allowed("example.com/path/foo/another/bar"));
        assert!(whitelist.is_allowed("example.com/path/segment/another/file.txt"));
        assert!(whitelist.is_allowed("example.com/path/x/another/y"));
        assert!(whitelist.is_allowed("example.com/path/anything/another/something"));

        // Should also work with longer segments where wildcards are
        assert!(whitelist.is_allowed("example.com/path/very/long/segment/another/more/stuff"));

        // Should not match if missing required path segments
        assert!(!whitelist.is_allowed("example.com/path/foo/another")); // Missing final segment
        assert!(!whitelist.is_allowed("example.com/path/another/bar")); // Missing middle segment
        assert!(!whitelist.is_allowed("example.com/path/foo/wrong/bar")); // Wrong middle segment
        assert!(!whitelist.is_allowed("example.com/wrong/foo/another/bar")); // Wrong first segment

        // Test more complex patterns
        whitelist.add("api.com/v*/users/*/profile");
        assert!(whitelist.is_allowed("api.com/v1/users/123/profile"));
        assert!(whitelist.is_allowed("api.com/v2/users/john/profile"));
        assert!(!whitelist.is_allowed("api.com/v1/users/123/settings"));

        // Test with leading/trailing wildcards
        whitelist.add("cdn.com/*/assets/*");
        whitelist.add("*.*.cdn.com/*/assets/*");
        assert!(whitelist.is_allowed("cdn.com/2024/assets/image.png"));
        assert!(whitelist.is_allowed("test.test.cdn.com/user/123/assets/doc.pdf"));

        let mut whitelist2 = Whitelist::new();

        // Test the specific case with multiple wildcards in path
        whitelist2.add("cdn.com/*/assets/*/");
        assert!(!whitelist2.is_allowed("cdn.com/test/assets/test/test"));
    }
}
