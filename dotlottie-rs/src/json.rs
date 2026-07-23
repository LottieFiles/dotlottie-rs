//! A minimal, allocation-conscious JSON DOM.

use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    Number(f64),
    String(Cow<'a, str>),
    Array(Vec<Value<'a>>),
    Object(Vec<(Cow<'a, str>, Value<'a>)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub message: &'static str,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "JSON error at byte {}: {}", self.offset, self.message)
    }
}

#[cfg_attr(not(feature = "dotlottie"), allow(dead_code))]
impl<'a> Value<'a> {
    /// Parse a complete JSON document.
    pub fn parse(input: &'a str) -> Result<Value<'a>, ParseError> {
        let mut p = Parser {
            s: input.as_bytes(),
            src: input,
            i: 0,
        };
        p.skip_ws();
        let v = p.value(0)?;
        p.skip_ws();
        if p.i != p.s.len() {
            return Err(p.err("trailing characters"));
        }
        Ok(v)
    }

    pub fn get(&self, key: &str) -> Option<&Value<'a>> {
        match self {
            Value::Object(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value<'a>> {
        match self {
            Value::Object(pairs) => pairs.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Insert or replace a key in an object.
    pub fn set(&mut self, key: &str, value: Value<'a>) {
        if let Value::Object(pairs) = self {
            if let Some(slot) = pairs.iter_mut().find(|(k, _)| k == key) {
                slot.1 = value;
            } else {
                pairs.push((Cow::Owned(key.to_owned()), value));
            }
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        self.as_f64().map(|n| n as f32)
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Value::Number(n) if n.fract() == 0.0 && *n >= 0.0 && *n <= u32::MAX as f64 => {
                Some(*n as u32)
            }
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value<'a>]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&[(Cow<'a, str>, Value<'a>)]> {
        match self {
            Value::Object(pairs) => Some(pairs),
            _ => None,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// `get(key)` then `as_str`.
    pub fn str_field(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    pub fn f32_field(&self, key: &str) -> Option<f32> {
        self.get(key).and_then(Value::as_f32)
    }

    pub fn u32_field(&self, key: &str) -> Option<u32> {
        self.get(key).and_then(Value::as_u32)
    }

    pub fn as_u8(&self) -> Option<u8> {
        u8::try_from(self.as_u32()?).ok()
    }

    pub fn u8_field(&self, key: &str) -> Option<u8> {
        self.get(key).and_then(Value::as_u8)
    }

    /// `opt` + owned-string: absent/null → `Some(None)`, non-string → `None`.
    pub fn opt_str_field(&self, key: &str) -> Option<Option<String>> {
        opt(self.get(key), |v| v.as_str().map(str::to_owned))
    }

    /// Serialize back to JSON text.
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(64);
        self.write(&mut out);
        out
    }

    pub fn write(&self, out: &mut String) {
        match self {
            Value::Null => out.push_str("null"),
            Value::Bool(true) => out.push_str("true"),
            Value::Bool(false) => out.push_str("false"),
            Value::Number(n) => write_number(*n, out),
            Value::String(s) => write_str(s, out),
            Value::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write(out);
                }
                out.push(']');
            }
            Value::Object(pairs) => {
                out.push('{');
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_str(k, out);
                    out.push(':');
                    v.write(out);
                }
                out.push('}');
            }
        }
    }
}

#[cfg_attr(not(feature = "dotlottie"), allow(dead_code))]
fn write_number(n: f64, out: &mut String) {
    if n.is_finite() {
        // Rust's Display for f64 prints the shortest round-trip form, which is
        // both compact and valid JSON (integers print without ".0").
        use core::fmt::Write;
        let _ = write!(out, "{n}");
    } else {
        // serde_json serializes non-finite floats as null.
        out.push_str("null");
    }
}

// Never widen an f32 to f64 before printing — 0.1f32 as f64 prints 17 digits.
pub fn write_f32(v: f32, out: &mut String) {
    if v.is_finite() {
        use core::fmt::Write;
        let _ = write!(out, "{v}");
    } else {
        out.push_str("null");
    }
}

pub fn write_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                use core::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Parse an optional field: absent key or JSON null → `Some(None)`,
/// present but unparsable → `None`.
pub fn opt<'a, T>(
    field: Option<&Value<'a>>,
    parse: impl FnOnce(&Value<'a>) -> Option<T>,
) -> Option<Option<T>> {
    match field {
        None | Some(Value::Null) => Some(None),
        Some(v) => parse(v).map(Some),
    }
}

pub fn array_of<'a, T>(v: &Value<'a>, parse: impl Fn(&Value<'a>) -> Option<T>) -> Option<Vec<T>> {
    v.as_array()?.iter().map(parse).collect()
}

pub fn f32_vec(v: &Value) -> Option<Vec<f32>> {
    array_of(v, Value::as_f32)
}

pub fn f32_array<const N: usize>(v: &Value) -> Option<[f32; N]> {
    let arr = v.as_array()?;
    if arr.len() != N {
        return None;
    }
    let mut out = [0.0; N];
    for (slot, item) in out.iter_mut().zip(arr) {
        *slot = item.as_f32()?;
    }
    Some(out)
}

/// Write `items` as a JSON array, one `f` call per element.
pub fn write_seq<'a, T: 'a>(
    out: &mut String,
    items: impl IntoIterator<Item = &'a T>,
    f: impl Fn(&'a T, &mut String),
) {
    out.push('[');
    let mut first = true;
    for item in items {
        if !first {
            out.push(',');
        }
        first = false;
        f(item, out);
    }
    out.push(']');
}

pub struct ObjWriter<'a> {
    out: &'a mut String,
    first: bool,
}

impl<'a> ObjWriter<'a> {
    pub fn new(out: &'a mut String) -> Self {
        out.push('{');
        Self { out, first: true }
    }

    /// Write `"key":` (with any needed comma) and return the buffer so the
    /// caller can write the value.
    pub fn field(&mut self, key: &str) -> &mut String {
        if !self.first {
            self.out.push(',');
        }
        self.first = false;
        write_str(key, self.out);
        self.out.push(':');
        self.out
    }

    pub fn finish(self) {
        self.out.push('}');
    }
}

const MAX_DEPTH: u32 = 128;

struct Parser<'a> {
    s: &'a [u8],
    src: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn err(&self, message: &'static str) -> ParseError {
        ParseError {
            offset: self.i,
            message,
        }
    }

    fn skip_ws(&mut self) {
        while let Some(&b) = self.s.get(self.i) {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' => self.i += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn expect(&mut self, b: u8, message: &'static str) -> Result<(), ParseError> {
        if self.peek() == Some(b) {
            self.i += 1;
            Ok(())
        } else {
            Err(self.err(message))
        }
    }

    fn value(&mut self, depth: u32) -> Result<Value<'a>, ParseError> {
        if depth >= MAX_DEPTH {
            return Err(self.err("maximum nesting depth exceeded"));
        }
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => Ok(Value::String(self.string()?)),
            Some(b't') => self.literal(b"true", Value::Bool(true)),
            Some(b'f') => self.literal(b"false", Value::Bool(false)),
            Some(b'n') => self.literal(b"null", Value::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.err("unexpected character")),
        }
    }

    fn literal(&mut self, lit: &'static [u8], v: Value<'a>) -> Result<Value<'a>, ParseError> {
        if self.s[self.i..].starts_with(lit) {
            self.i += lit.len();
            Ok(v)
        } else {
            Err(self.err("invalid literal"))
        }
    }

    fn number(&mut self) -> Result<Value<'a>, ParseError> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        match self.peek() {
            Some(b'0') => {
                self.i += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(self.err("invalid number"));
                }
            }
            Some(b'1'..=b'9') => {
                self.i += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.i += 1;
                }
            }
            _ => return Err(self.err("invalid number")),
        }
        if self.peek() == Some(b'.') {
            self.i += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.err("invalid number"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.i += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.i += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.i += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.err("invalid number"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.i += 1;
            }
        }
        let text = &self.src[start..self.i];
        let n: f64 = text.parse().map_err(|_| ParseError {
            offset: start,
            message: "invalid number",
        })?;
        if !n.is_finite() {
            return Err(ParseError {
                offset: start,
                message: "invalid number",
            });
        }
        Ok(Value::Number(n))
    }

    fn string(&mut self) -> Result<Cow<'a, str>, ParseError> {
        self.expect(b'"', "expected string")?;
        let start = self.i;
        // Fast path: scan for a string without escapes and borrow it.
        loop {
            match self.peek() {
                Some(b'"') => {
                    let s = &self.src[start..self.i];
                    self.i += 1;
                    return Ok(Cow::Borrowed(s));
                }
                Some(b'\\') => break, // slow path below
                Some(b) if b < 0x20 => return Err(self.err("control character in string")),
                Some(_) => self.i += 1,
                None => return Err(self.err("unterminated string")),
            }
        }
        // Slow path: unescape into an owned buffer.
        let mut out = String::with_capacity(self.i - start + 16);
        out.push_str(&self.src[start..self.i]);
        loop {
            match self.peek() {
                Some(b'"') => {
                    self.i += 1;
                    return Ok(Cow::Owned(out));
                }
                Some(b'\\') => {
                    self.i += 1;
                    match self.peek() {
                        Some(b'"') => out.push('"'),
                        Some(b'\\') => out.push('\\'),
                        Some(b'/') => out.push('/'),
                        Some(b'b') => out.push('\u{08}'),
                        Some(b'f') => out.push('\u{0c}'),
                        Some(b'n') => out.push('\n'),
                        Some(b'r') => out.push('\r'),
                        Some(b't') => out.push('\t'),
                        Some(b'u') => {
                            self.i += 1;
                            let hi = self.hex4()?;
                            let c = if (0xD800..0xDC00).contains(&hi) {
                                // Surrogate pair.
                                if self.s[self.i..].starts_with(b"\\u") {
                                    self.i += 2;
                                    let lo = self.hex4()?;
                                    if !(0xDC00..0xE000).contains(&lo) {
                                        return Err(self.err("invalid low surrogate"));
                                    }
                                    let c = 0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                                    char::from_u32(c)
                                        .ok_or_else(|| self.err("invalid surrogate pair"))?
                                } else {
                                    return Err(self.err("unpaired surrogate"));
                                }
                            } else if (0xDC00..0xE000).contains(&hi) {
                                return Err(self.err("unpaired surrogate"));
                            } else {
                                char::from_u32(hi).ok_or_else(|| self.err("invalid \\u escape"))?
                            };
                            out.push(c);
                            continue; // hex4 already advanced past the digits
                        }
                        _ => return Err(self.err("invalid escape sequence")),
                    }
                    self.i += 1;
                }
                Some(b) if b < 0x20 => return Err(self.err("control character in string")),
                Some(_) => {
                    // Copy a run of plain bytes (UTF-8 passes through verbatim).
                    let run = self.i;
                    while matches!(self.peek(), Some(b) if b != b'"' && b != b'\\' && b >= 0x20) {
                        self.i += 1;
                    }
                    out.push_str(&self.src[run..self.i]);
                }
                None => return Err(self.err("unterminated string")),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, ParseError> {
        let mut v = 0u32;
        for _ in 0..4 {
            let d = match self.peek() {
                Some(b @ b'0'..=b'9') => (b - b'0') as u32,
                Some(b @ b'a'..=b'f') => (b - b'a' + 10) as u32,
                Some(b @ b'A'..=b'F') => (b - b'A' + 10) as u32,
                _ => return Err(self.err("invalid hex digit")),
            };
            v = v * 16 + d;
            self.i += 1;
        }
        Ok(v)
    }

    fn array(&mut self, depth: u32) -> Result<Value<'a>, ParseError> {
        self.expect(b'[', "expected array")?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Value::Array(items));
        }
        loop {
            self.skip_ws();
            items.push(self.value(depth + 1)?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Value::Array(items));
                }
                _ => return Err(self.err("expected ',' or ']'")),
            }
        }
    }

    fn object(&mut self, depth: u32) -> Result<Value<'a>, ParseError> {
        self.expect(b'{', "expected object")?;
        let mut pairs = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Value::Object(pairs));
        }
        loop {
            self.skip_ws();
            let key = self.string()?;
            self.skip_ws();
            self.expect(b':', "expected ':'")?;
            self.skip_ws();
            let value = self.value(depth + 1)?;
            // Last occurrence of a duplicate key wins, matching serde_json.
            if let Some(slot) = pairs.iter_mut().find(|(k, _)| *k == key) {
                slot.1 = value;
            } else {
                pairs.push((key, value));
            }
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Value::Object(pairs));
                }
                _ => return Err(self.err("expected ',' or '}'")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scalars() {
        assert_eq!(Value::parse("null").unwrap(), Value::Null);
        assert_eq!(Value::parse("true").unwrap(), Value::Bool(true));
        assert_eq!(Value::parse(" -12.5e2 ").unwrap(), Value::Number(-1250.0));
        assert_eq!(Value::parse("\"hi\"").unwrap().as_str(), Some("hi"));
    }

    #[test]
    fn zero_copy_strings_borrow() {
        let v = Value::parse("\"plain\"").unwrap();
        match v {
            Value::String(Cow::Borrowed(s)) => assert_eq!(s, "plain"),
            other => panic!("expected borrowed string, got {other:?}"),
        }
    }

    #[test]
    fn unescapes_strings() {
        let v = Value::parse(r#""a\nbé😀""#).unwrap();
        assert_eq!(v.as_str(), Some("a\nbé😀"));
    }

    #[test]
    fn parses_nested_structures() {
        let v = Value::parse(r#"{"a":[1,2,{"b":false}],"c":{"d":null}}"#).unwrap();
        assert_eq!(v.get("a").unwrap().as_array().unwrap().len(), 3);
        assert_eq!(v.get("c").unwrap().get("d"), Some(&Value::Null));
    }

    #[test]
    fn roundtrips() {
        let src = r#"{"a":[1,2.5,{"b":false}],"s":"q\"uote","n":null}"#;
        let v = Value::parse(src).unwrap();
        let out = v.to_json();
        assert_eq!(Value::parse(&out).unwrap(), v);
    }

    #[test]
    fn rejects_garbage() {
        assert!(Value::parse("{").is_err());
        assert!(Value::parse("[1,]").is_err());
        assert!(Value::parse("nul").is_err());
        assert!(Value::parse("1 2").is_err());
        let deep = "[".repeat(200) + &"]".repeat(200);
        assert!(Value::parse(&deep).is_err());
    }

    #[test]
    fn duplicate_keys_last_wins() {
        let v = Value::parse(r#"{"a":1,"b":true,"a":2}"#).unwrap();
        assert_eq!(v.get("a").unwrap().as_f64(), Some(2.0));
        assert_eq!(v.as_object().unwrap().len(), 2);
        assert_eq!(v.to_json(), r#"{"a":2,"b":true}"#);
    }

    #[test]
    fn non_finite_writes_null() {
        assert_eq!(Value::Number(f64::NAN).to_json(), "null");
        assert_eq!(Value::Number(f64::INFINITY).to_json(), "null");
        let mut out = String::new();
        write_f32(f32::NEG_INFINITY, &mut out);
        assert_eq!(out, "null");
    }

    #[test]
    fn depth_limit_matches_serde_json() {
        let ok = "[".repeat(127) + "1" + &"]".repeat(127);
        assert!(Value::parse(&ok).is_ok());
        let too_deep = "[".repeat(128) + "1" + &"]".repeat(128);
        assert!(Value::parse(&too_deep).is_err());
    }

    #[test]
    fn set_replaces_and_inserts() {
        let mut v = Value::parse(r#"{"a":1}"#).unwrap();
        v.set("a", Value::Number(2.0));
        v.set("b", Value::Bool(true));
        assert_eq!(v.get("a").unwrap().as_f64(), Some(2.0));
        assert_eq!(v.get("b").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn as_u32_is_integral_only() {
        assert_eq!(Value::parse("7").unwrap().as_u32(), Some(7));
        assert_eq!(Value::parse("7.0").unwrap().as_u32(), Some(7));
        assert_eq!(Value::parse("7.5").unwrap().as_u32(), None);
        assert_eq!(Value::parse("-1").unwrap().as_u32(), None);
    }

    #[test]
    fn as_bool_is_strict() {
        assert_eq!(Value::parse("true").unwrap().as_bool(), Some(true));
        assert_eq!(Value::parse("1").unwrap().as_bool(), None);
        assert_eq!(Value::parse("0").unwrap().as_bool(), None);
    }

    #[test]
    fn opt_distinguishes_absent_from_invalid() {
        let v = Value::parse(r#"{"a":null,"b":"x"}"#).unwrap();
        assert_eq!(opt(v.get("missing"), Value::as_f32), Some(None));
        assert_eq!(opt(v.get("a"), Value::as_f32), Some(None));
        assert_eq!(opt(v.get("b"), Value::as_f32), None);
        assert_eq!(
            opt(v.get("b"), |b| b.as_str().map(str::to_owned)),
            Some(Some("x".to_string()))
        );
    }

    #[test]
    fn obj_writer_commas_and_escapes() {
        let mut out = String::new();
        let mut o = ObjWriter::new(&mut out);
        write_str("a\"b", o.field("k"));
        write_f32(0.1, o.field("n"));
        o.finish();
        assert_eq!(out, r#"{"k":"a\"b","n":0.1}"#);
    }

    #[test]
    fn write_f32_stays_in_f32_precision() {
        let mut out = String::new();
        write_f32(0.1_f32, &mut out);
        assert_eq!(out, "0.1");
    }

    #[test]
    fn rejects_non_json_numbers() {
        for bad in [
            "1.", "-.5", "01", "1.e5", ".5", "-", "1e", "1e+", "00", "1e999",
        ] {
            assert!(Value::parse(bad).is_err(), "should reject {bad}");
        }
        assert_eq!(Value::parse("0").unwrap(), Value::Number(0.0));
        assert_eq!(Value::parse("0.5e-3").unwrap(), Value::Number(0.0005));
        assert_eq!(Value::parse("-0").unwrap(), Value::Number(-0.0));
    }
}
