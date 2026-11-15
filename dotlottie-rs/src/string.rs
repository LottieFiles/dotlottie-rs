//! `DotString` is an immutable string whose `Clone` bumps a refcount. It
//! derefs to `str`. With the `c_api` feature, `as_ptr()` returns a
//! null-terminated `*const c_char` without copying.
//!
//! # Representation
//!
//! - With `c_api`, storage is an `Arc<[u8]>` that ends in a null byte, has no
//!   interior nulls, and is valid UTF-8. `as_str` and `as_c_str` read it
//!   without copying or revalidating.
//! - Without `c_api`, storage is an `Arc<str>` with no null terminator.
//!
//! `Clone` never allocates in either configuration. `c_api` adds `as_c_str`,
//! `as_ptr` and the `CString`/`CStr` conversions. The rest of the API is the
//! same in both.

use std::sync::Arc;

#[cfg(feature = "c_api")]
use std::ffi::{CStr, CString};

/// Returned when the input to a `DotString` constructor contains a null byte.
/// Only `c_api` builds return it, because they append a null terminator.
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

    /// Every constructor upholds these invariants:
    ///   1. `bytes.last() == Some(&0)`, a trailing null terminator.
    ///   2. `bytes[..len-1]` contains no `0`, so there are no interior nulls.
    ///   3. `bytes[..len-1]` is valid UTF-8, which `as_str` relies on.
    #[derive(Clone)]
    pub(crate) struct Storage {
        pub bytes: Arc<[u8]>,
    }
}

#[cfg(not(feature = "c_api"))]
mod repr {
    use std::sync::Arc;

    #[derive(Clone)]
    pub(crate) struct Storage {
        pub inner: Arc<str>,
    }
}

/// Immutable string with refcounted storage. See the module docs.
#[derive(Clone)]
pub struct DotString {
    storage: repr::Storage,
}

impl DotString {
    /// Copies `s` into a new `DotString`. With `c_api`, panics if `s`
    /// contains a null byte.
    pub fn new(s: &str) -> Self {
        Self::try_new(s).expect("DotString: interior null byte not allowed")
    }

    /// Copies `s` into a new `DotString`. With `c_api`, returns
    /// `Err(InteriorNul)` if `s` contains a null byte.
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

    /// Converts a `String` into a `DotString`. With `c_api`, panics if `s`
    /// contains a null byte.
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

    /// Converts a `CString` into a `DotString`. Debug builds assert that the
    /// bytes are valid UTF-8.
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

    /// Returns a shared empty `DotString`. The first call allocates, and later
    /// calls return a clone.
    pub fn empty() -> Self {
        static EMPTY: std::sync::OnceLock<DotString> = std::sync::OnceLock::new();
        EMPTY.get_or_init(|| DotString::new("")).clone()
    }

    /// Returns the contents as `&str` without copying or revalidating.
    #[inline]
    pub fn as_str(&self) -> &str {
        #[cfg(feature = "c_api")]
        {
            let len = self.storage.bytes.len();
            debug_assert!(len >= 1 && self.storage.bytes[len - 1] == 0);
            // SAFETY: constructors uphold invariant 3, so the bytes before
            // the trailing null are valid UTF-8.
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
    /// Returns the contents as `&CStr`. The stored bytes already end in a
    /// null, so this borrows them without copying.
    #[inline]
    pub fn as_c_str(&self) -> &CStr {
        // SAFETY: constructors uphold invariants 1 and 2, so the last byte
        // is the only null.
        unsafe { CStr::from_bytes_with_nul_unchecked(&self.storage.bytes) }
    }

    /// Returns a pointer to the null-terminated bytes. The pointer stays valid
    /// while this `DotString` or any clone of it is alive, because clones
    /// share one `Arc`.
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
        // Clones and interned strings share an Arc, so this skips the byte compare.
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

/// Deduplicating cache for `DotString`s. Interning the same string again
/// returns a clone of the stored `DotString` and does not allocate.
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

    /// Returns a `DotString` for `s`. The first call for a given string
    /// allocates, and later calls return a clone of the stored one.
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
        // Clones share the same Arc.
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

        // a and b share storage, c does not.
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
        // Hits the Arc::ptr_eq check in PartialEq.
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
}
