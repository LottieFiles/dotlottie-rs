use crate::json::{opt, write_str, ObjWriter, Value};
use std::fmt::Write as _;

#[derive(Debug, Clone)]
pub struct ImageSlot {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub directory: Option<String>,
    pub path: Option<String>,
    pub embed: Option<u8>,
}

impl ImageSlot {
    /// Resolve a theme `src` into an image slot by its prefix
    /// (`data:` → embedded, `http(s)://` → remote, otherwise a package `i/` file).
    pub fn from_src(src: String) -> Self {
        if src.starts_with("data:") {
            Self::from_data_url(src)
        } else if src.starts_with("http://") || src.starts_with("https://") {
            Self::from_url(src)
        } else {
            Self::from_path(src)
        }
    }

    pub fn from_path(path: String) -> Self {
        let filename = path.split('/').next_back().unwrap_or(&path).to_string();
        let dir = path.rsplit_once('/').map(|x| x.0.to_string());

        Self {
            width: None,
            height: None,
            directory: dir,
            path: Some(filename),
            embed: Some(0),
        }
    }

    pub fn from_data_url(data_url: String) -> Self {
        Self {
            width: None,
            height: None,
            directory: None,
            path: Some(data_url),
            embed: Some(1),
        }
    }

    pub fn from_url(url: String) -> Self {
        Self {
            width: None,
            height: None,
            directory: None,
            path: Some(url),
            embed: Some(0),
        }
    }

    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn is_embedded(&self) -> bool {
        matches!(self.path.as_deref(), Some(p) if p.starts_with("data:"))
    }

    pub fn is_remote(&self) -> bool {
        matches!(self.path.as_deref(), Some(p) if p.starts_with("http://") || p.starts_with("https://"))
    }

    pub fn file_name(&self) -> Option<&str> {
        if self.is_embedded() || self.is_remote() {
            return None;
        }

        self.path.as_deref().filter(|name| !name.is_empty())
    }

    /// ThorVG discriminates image assets from audio on `w`/`h` being non-zero,
    /// so a zero dimension is as good as an absent one.
    pub fn has_dimensions(&self) -> bool {
        matches!((self.width, self.height), (Some(w), Some(h)) if w > 0 && h > 0)
    }

    pub fn inline(&mut self, data_url: String) {
        self.directory = None;
        self.path = Some(data_url);
        self.embed = Some(1);
    }
}

pub(crate) fn image_slot_from_json(v: &Value) -> Option<ImageSlot> {
    if !v.is_object() {
        return None;
    }
    Some(ImageSlot {
        width: opt(v.get("w"), Value::as_u32)?,
        height: opt(v.get("h"), Value::as_u32)?,
        directory: v.opt_str_field("u")?,
        path: v.opt_str_field("p")?,
        embed: opt(v.get("e"), Value::as_u8)?,
    })
}

pub(crate) fn write_image_slot(img: &ImageSlot, out: &mut String) {
    let mut o = ObjWriter::new(out);
    if let Some(w) = img.width {
        let _ = write!(o.field("w"), "{w}");
    }
    if let Some(h) = img.height {
        let _ = write!(o.field("h"), "{h}");
    }
    if let Some(u) = &img.directory {
        write_str(u, o.field("u"));
    }
    if let Some(p) = &img.path {
        write_str(p, o.field("p"));
    }
    if let Some(e) = img.embed {
        let _ = write!(o.field("e"), "{e}");
    }
    o.finish();
}

#[cfg(test)]
mod tests {
    use super::ImageSlot;

    #[test]
    fn data_url_is_embedded_and_has_no_file_name() {
        let slot = ImageSlot::from_src("data:image/png;base64,AAAA".to_string());

        assert!(slot.is_embedded());
        assert!(!slot.is_remote());
        assert_eq!(slot.file_name(), None);
    }

    #[test]
    fn http_url_is_remote_and_has_no_file_name() {
        for src in ["http://example.com/a.png", "https://example.com/a.png"] {
            let slot = ImageSlot::from_src(src.to_string());

            assert!(slot.is_remote());
            assert!(!slot.is_embedded());
            assert_eq!(slot.file_name(), None);
        }
    }

    #[test]
    fn package_paths_all_reduce_to_the_bare_file_name() {
        for src in ["logo.png", "i/logo.png", "/i/logo.png"] {
            let slot = ImageSlot::from_src(src.to_string());

            assert_eq!(slot.file_name(), Some("logo.png"), "src = {src}");
        }
    }

    #[test]
    fn inline_replaces_a_package_reference_with_embedded_bytes() {
        let mut slot = ImageSlot::from_src("/i/logo.png".to_string());
        assert_eq!(slot.file_name(), Some("logo.png"));

        slot.inline("data:image/png;base64,AAAA".to_string());

        assert!(slot.is_embedded());
        assert_eq!(slot.directory, None);
        assert_eq!(slot.file_name(), None);
        assert_eq!(slot.path.as_deref(), Some("data:image/png;base64,AAAA"));
    }

    #[test]
    fn an_empty_src_has_no_file_name() {
        let slot = ImageSlot::from_src(String::new());

        assert!(!slot.is_embedded());
        assert!(!slot.is_remote());
        assert_eq!(slot.file_name(), None);
    }

    #[test]
    fn dimensions_are_reported_only_when_both_are_present() {
        let slot = ImageSlot::from_src("logo.png".to_string());
        assert!(!slot.has_dimensions());

        assert!(slot.with_dimensions(250, 167).has_dimensions());
    }

    #[test]
    fn a_zero_dimension_does_not_count_as_a_dimension() {
        let slot = ImageSlot::from_src("logo.png".to_string());

        assert!(!slot.clone().with_dimensions(0, 0).has_dimensions());
        assert!(!slot.clone().with_dimensions(250, 0).has_dimensions());
        assert!(!slot.with_dimensions(0, 167).has_dimensions());
    }

    #[test]
    fn an_embed_flag_does_not_make_a_bare_file_name_embedded() {
        let mut slot = ImageSlot::from_src("logo.png".to_string());
        slot.embed = Some(1);

        assert!(!slot.is_embedded());
        assert_eq!(slot.file_name(), Some("logo.png"));
    }
}
