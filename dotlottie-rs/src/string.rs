//! `DotString` — a cheaply cloneable, immutable string type that is ergonomic
//! in Rust (`Deref<Target = str>`) and zero-cost at the FFI boundary
//! (`as_ptr()` returns a stable `*const c_char`) when the `c_api` feature is
//! enabled.
//!
//! # Representation
//!
//! - `c_api` on:  `Arc<[u8]>` with invariants (trailing null, no interior
//!   nulls, valid UTF-8). Read both as `&str` (zero cost) and `&CStr` (zero
//!   cost) without revalidation.
//! - `c_api` off: `Arc<str>` — no null-terminator overhead.
//!
//! In both cases, `Clone` is a refcount bump (no allocation), and the public
//! API is identical: Rust users see a `Deref<Target = str>` wrapper.

use std::sync::Arc;

#[cfg(feature = "c_api")]
use std::ffi::{CStr, CString};

/// Error returned when constructing a `DotString` from bytes that contain an
/// interior null byte (only possible when the `c_api` feature is enabled,
/// since a trailing null is required).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteriorNul;

impl std::fmt::Display for InteriorNul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DotString: interior null byte not allowed")
    }
}

impl std::error::Error for InteriorNul {}

#[cfg(feature = "c_api")]
mod repr {
    use std::sync::Arc;

    /// Storage invariants (upheld at every construction site):
    ///   1. `bytes.last() == Some(&0)`           — trailing null terminator
    ///   2. `bytes[..len-1]` contains no `0`     — no interior nulls
    ///   3. `bytes[..len-1]` is valid UTF-8      — so `as_str` is safe
    #[derive(Clone)]
    pub(super) struct Storage {
        pub bytes: Arc<[u8]>,
    }
}

#[cfg(not(feature = "c_api"))]
mod repr {
    use std::sync::Arc;

    #[derive(Clone)]
    pub(super) struct Storage {
        pub inner: Arc<str>,
    }
}

/// Immutable, refcount-shared string. See module docs.
#[derive(Clone)]
pub struct DotString {
    storage: repr::Storage,
}

impl DotString {
    /// Construct from a `&str`. Panics if the string contains an interior
    /// null byte when the `c_api` feature is enabled.
    pub fn new(s: &str) -> Self {
        Self::try_new(s).expect("DotString: interior null byte not allowed")
    }

    /// Construct from a `&str`, returning `Err(InteriorNul)` if the input
    /// contains a null byte (c_api builds only).
    #[allow(clippy::unnecessary_wraps)]
    pub fn try_new(s: &str) -> Result<Self, InteriorNul> {
        #[cfg(feature = "c_api")]
        {
            if s.as_bytes().contains(&0) {
                return Err(InteriorNul);
            }
            let mut v = Vec::with_capacity(s.len() + 1);
            v.extend_from_slice(s.as_bytes());
            v.push(0);
            Ok(DotString {
                storage: repr::Storage {
                    bytes: Arc::from(v.into_boxed_slice()),
                },
            })
        }
        #[cfg(not(feature = "c_api"))]
        {
            Ok(DotString {
                storage: repr::Storage {
                    inner: Arc::from(s),
                },
            })
        }
    }

    /// Take ownership of a `String`. Zero-copy when `c_api` is off; when on,
    /// reuses the `String`'s allocation and appends a null terminator.
    pub fn from_string(s: String) -> Self {
        Self::try_from_string(s).expect("DotString: interior null byte not allowed")
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn try_from_string(s: String) -> Result<Self, InteriorNul> {
        #[cfg(feature = "c_api")]
        {
            if s.as_bytes().contains(&0) {
                return Err(InteriorNul);
            }
            let mut v = s.into_bytes();
            v.push(0);
            Ok(DotString {
                storage: repr::Storage {
                    bytes: Arc::from(v.into_boxed_slice()),
                },
            })
        }
        #[cfg(not(feature = "c_api"))]
        {
            Ok(DotString {
                storage: repr::Storage {
                    inner: Arc::from(s.into_boxed_str()),
                },
            })
        }
    }

    /// Zero-copy transfer from a `CString` (c_api only).
    #[cfg(feature = "c_api")]
    pub fn from_cstring(c: CString) -> Self {
        let bytes = c.into_bytes_with_nul();
        debug_assert!(
            std::str::from_utf8(&bytes[..bytes.len() - 1]).is_ok(),
            "DotString from non-UTF-8 CString"
        );
        DotString {
            storage: repr::Storage {
                bytes: Arc::from(bytes.into_boxed_slice()),
            },
        }
    }

    /// Shared empty `DotString`. Allocates at most once per process.
    pub fn empty() -> Self {
        static EMPTY: std::sync::OnceLock<DotString> = std::sync::OnceLock::new();
        EMPTY.get_or_init(|| DotString::new("")).clone()
    }

    /// View as a `&str`. Zero cost in both feature configurations.
    #[inline]
    pub fn as_str(&self) -> &str {
        #[cfg(feature = "c_api")]
        {
            let len = self.storage.bytes.len();
            debug_assert!(len >= 1 && self.storage.bytes[len - 1] == 0);
            // SAFETY: invariants 2 and 3 are upheld at every construction
            // site, so the bytes before the trailing null are valid UTF-8
            // with no interior nulls.
            unsafe { std::str::from_utf8_unchecked(&self.storage.bytes[..len - 1]) }
        }
        #[cfg(not(feature = "c_api"))]
        {
            &self.storage.inner
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(feature = "c_api")]
impl DotString {
    /// View as a `&CStr`. Zero cost — returns a view over the same underlying
    /// bytes, which include the trailing null by invariant.
    #[inline]
    pub fn as_c_str(&self) -> &CStr {
        // SAFETY: invariants 1 (trailing null) and 2 (no interior null) are
        // upheld at every construction site.
        unsafe { CStr::from_bytes_with_nul_unchecked(&self.storage.bytes) }
    }

    /// Raw pointer to the null-terminated bytes. Valid for the lifetime of
    /// this `DotString` (and any `clone()`s thereof, since they share
    /// storage via `Arc`).
    #[inline]
    pub fn as_ptr(&self) -> *const std::os::raw::c_char {
        self.storage.bytes.as_ptr() as *const _
    }
}

impl std::ops::Deref for DotString {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for DotString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for DotString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Default for DotString {
    fn default() -> Self {
        DotString::empty()
    }
}

impl PartialEq for DotString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // Fast path: same Arc → same content. O(1) for interned strings.
        #[cfg(feature = "c_api")]
        if Arc::ptr_eq(&self.storage.bytes, &other.storage.bytes) {
            return true;
        }
        #[cfg(not(feature = "c_api"))]
        if Arc::ptr_eq(&self.storage.inner, &other.storage.inner) {
            return true;
        }
        self.as_str() == other.as_str()
    }
}

impl Eq for DotString {}

impl PartialEq<str> for DotString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for DotString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<DotString> for str {
    fn eq(&self, other: &DotString) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<DotString> for &str {
    fn eq(&self, other: &DotString) -> bool {
        *self == other.as_str()
    }
}

impl PartialOrd for DotString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DotString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl std::hash::Hash for DotString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl std::fmt::Debug for DotString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

impl std::fmt::Display for DotString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_str(), f)
    }
}

impl From<&str> for DotString {
    fn from(s: &str) -> Self {
        DotString::new(s)
    }
}

impl From<String> for DotString {
    fn from(s: String) -> Self {
        DotString::from_string(s)
    }
}

impl From<&String> for DotString {
    fn from(s: &String) -> Self {
        DotString::new(s)
    }
}

impl From<Box<str>> for DotString {
    fn from(s: Box<str>) -> Self {
        DotString::from_string(String::from(s))
    }
}

#[cfg(feature = "c_api")]
impl From<CString> for DotString {
    fn from(c: CString) -> Self {
        DotString::from_cstring(c)
    }
}

#[cfg(feature = "c_api")]
impl From<&CStr> for DotString {
    fn from(c: &CStr) -> Self {
        let s = c.to_str().expect("DotString from non-UTF-8 CStr");
        DotString::new(s)
    }
}

impl<'de> serde::Deserialize<'de> for DotString {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = <std::borrow::Cow<'de, str> as serde::Deserialize>::deserialize(d)?;
        DotString::try_new(&s).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for DotString {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

/// Small per-component dedup cache. Repeated interns of the same string
/// return clones of the same `DotString` — refcount bumps, no allocation.
#[derive(Default)]
pub struct DotStringInterner {
    table: rustc_hash::FxHashMap<Box<str>, DotString>,
}

impl DotStringInterner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            table: rustc_hash::FxHashMap::with_capacity_and_hasher(cap, Default::default()),
        }
    }

    /// Return a `DotString` for `s`, reusing storage if already interned.
    /// First-time: one allocation. Subsequent: `Arc::clone` (refcount bump).
    pub fn intern(&mut self, s: &str) -> DotString {
        if let Some(existing) = self.table.get(s) {
            return existing.clone();
        }
        let ds = DotString::new(s);
        self.table.insert(s.into(), ds.clone());
        ds
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_as_str() {
        let s = DotString::new("hello");
        assert_eq!(s.as_str(), "hello");
        assert_eq!(&*s, "hello");
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());
    }

    #[test]
    fn empty_string() {
        let s = DotString::empty();
        assert_eq!(s.as_str(), "");
        assert!(s.is_empty());
    }

    #[test]
    fn cheap_clone_shares_storage() {
        let a = DotString::new("shared");
        let b = a.clone();
        // Pointer equality — the Arc is shared.
        #[cfg(feature = "c_api")]
        assert!(Arc::ptr_eq(&a.storage.bytes, &b.storage.bytes));
        #[cfg(not(feature = "c_api"))]
        assert!(Arc::ptr_eq(&a.storage.inner, &b.storage.inner));
    }

    #[test]
    fn from_string_preserves_content() {
        let built = format!("state: {}", "running");
        let s = DotString::from_string(built);
        assert_eq!(&*s, "state: running");
    }

    #[test]
    fn equality_with_str_both_ways() {
        let s = DotString::new("idle");
        assert_eq!(s, "idle");
        assert_eq!("idle", s);
        assert_eq!(&s, "idle");
        assert_ne!(s, "playing");
    }

    #[test]
    fn display_and_debug() {
        let s = DotString::new("event");
        assert_eq!(format!("{s}"), "event");
        assert_eq!(format!("{s:?}"), "\"event\"");
    }

    #[test]
    fn interner_dedupes() {
        let mut interner = DotStringInterner::new();
        let a = interner.intern("alpha");
        let b = interner.intern("alpha");
        let c = interner.intern("beta");

        assert_eq!(interner.len(), 2);
        assert_eq!(a, b);
        assert_ne!(a, c);

        // a and b share storage; c is distinct.
        #[cfg(feature = "c_api")]
        {
            assert!(Arc::ptr_eq(&a.storage.bytes, &b.storage.bytes));
            assert!(!Arc::ptr_eq(&a.storage.bytes, &c.storage.bytes));
        }
        #[cfg(not(feature = "c_api"))]
        {
            assert!(Arc::ptr_eq(&a.storage.inner, &b.storage.inner));
            assert!(!Arc::ptr_eq(&a.storage.inner, &c.storage.inner));
        }
    }

    #[test]
    fn interned_equality_fast_path() {
        let mut interner = DotStringInterner::new();
        let a = interner.intern("same");
        let b = interner.intern("same");
        // Triggers Arc::ptr_eq fast path in PartialEq.
        assert_eq!(a, b);
    }

    #[test]
    fn ordering_follows_str() {
        let a = DotString::new("alpha");
        let b = DotString::new("beta");
        assert!(a < b);
    }

    #[test]
    fn hash_matches_str() {
        use std::collections::HashMap;
        let mut m: HashMap<DotString, i32> = HashMap::new();
        m.insert(DotString::new("key"), 42);

        // Borrow<str> allows &str lookups.
        assert_eq!(m.get("key"), Some(&42));
    }

    #[test]
    fn default_is_empty() {
        let d: DotString = Default::default();
        assert_eq!(&*d, "");
    }

    #[cfg(feature = "c_api")]
    #[test]
    fn rejects_interior_null() {
        assert!(DotString::try_new("has\0null").is_err());
        assert!(DotString::try_from_string("also\0bad".to_string()).is_err());
    }

    #[cfg(feature = "c_api")]
    #[test]
    fn cstr_roundtrip() {
        let s = DotString::new("ffi");
        let c = s.as_c_str();
        assert_eq!(c.to_bytes(), b"ffi");
        assert_eq!(c.to_bytes_with_nul(), b"ffi\0");
    }

    #[cfg(feature = "c_api")]
    #[test]
    fn as_ptr_is_stable_and_null_terminated() {
        let s = DotString::new("stable");
        let ptr = s.as_ptr();
        // Round-trip through CStr from the pointer.
        let back = unsafe { std::ffi::CStr::from_ptr(ptr) };
        assert_eq!(back.to_bytes(), b"stable");
    }

    #[cfg(feature = "c_api")]
    #[test]
    fn from_cstring_zero_copy() {
        let c = std::ffi::CString::new("moved").unwrap();
        let s = DotString::from(c);
        assert_eq!(&*s, "moved");
    }

    #[test]
    fn serde_roundtrip() {
        let s = DotString::new("serialized");
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"serialized\"");
        let back: DotString = serde_json::from_str(&json).unwrap();
        assert_eq!(back, "serialized");
    }
}
