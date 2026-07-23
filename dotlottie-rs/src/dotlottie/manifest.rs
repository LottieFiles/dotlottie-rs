use crate::json::{array_of, opt, write_seq, write_str, ObjWriter, Value};

#[derive(Debug, Clone)]
pub struct ManifestInitial {
    pub animation: Option<String>,
    pub state_machine: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ManifestTheme {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ManifestStateMachine {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ManifestAnimation {
    pub id: String,
    pub name: Option<String>,
    pub themes: Option<Vec<String>>,
    pub background: Option<String>,
    pub initial_theme: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub version: Option<String>,
    pub generator: Option<String>,

    pub initial: Option<ManifestInitial>,

    pub animations: Vec<ManifestAnimation>,
    pub themes: Option<Vec<ManifestTheme>>,
    pub state_machines: Option<Vec<ManifestStateMachine>>,
}

fn id_name<T>(v: &Value, make: impl Fn(String, Option<String>) -> T) -> Option<T> {
    Some(make(
        v.str_field("id")?.to_owned(),
        v.opt_str_field("name")?,
    ))
}

fn write_opt_str(out: &mut String, s: &Option<String>) {
    match s {
        Some(s) => write_str(s, out),
        None => out.push_str("null"),
    }
}

impl Manifest {
    pub fn from_json(s: &str) -> Option<Manifest> {
        let root = Value::parse(s).ok()?;
        Self::from_value(&root)
    }

    fn from_value(root: &Value) -> Option<Manifest> {
        let initial = opt(root.get("initial"), |v| {
            if !v.is_object() {
                return None;
            }
            Some(ManifestInitial {
                animation: v.opt_str_field("animation")?,
                state_machine: v.opt_str_field("stateMachine")?,
            })
        })?;

        let animations = array_of(root.get("animations")?, |a| {
            Some(ManifestAnimation {
                id: a.str_field("id")?.to_owned(),
                name: a.opt_str_field("name")?,
                themes: opt(a.get("themes"), |t| {
                    array_of(t, |s| s.as_str().map(str::to_owned))
                })?,
                background: a.opt_str_field("background")?,
                initial_theme: a.opt_str_field("initialTheme")?,
            })
        })?;

        Some(Manifest {
            version: root.opt_str_field("version")?,
            generator: root.opt_str_field("generator")?,
            initial,
            animations,
            themes: opt(root.get("themes"), |t| {
                array_of(t, |v| id_name(v, |id, name| ManifestTheme { id, name }))
            })?,
            state_machines: opt(root.get("stateMachines"), |s| {
                array_of(s, |v| {
                    id_name(v, |id, name| ManifestStateMachine { id, name })
                })
            })?,
        })
    }

    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(256);
        let mut o = ObjWriter::new(&mut out);
        write_opt_str(o.field("version"), &self.version);
        write_opt_str(o.field("generator"), &self.generator);
        {
            let out = o.field("initial");
            match &self.initial {
                None => out.push_str("null"),
                Some(i) => {
                    let mut io = ObjWriter::new(out);
                    write_opt_str(io.field("animation"), &i.animation);
                    write_opt_str(io.field("stateMachine"), &i.state_machine);
                    io.finish();
                }
            }
        }
        write_seq(o.field("animations"), &self.animations, |a, out| {
            let mut ao = ObjWriter::new(out);
            write_str(&a.id, ao.field("id"));
            write_opt_str(ao.field("name"), &a.name);
            {
                let out = ao.field("themes");
                match &a.themes {
                    None => out.push_str("null"),
                    Some(themes) => write_seq(out, themes, |t, out| write_str(t, out)),
                }
            }
            write_opt_str(ao.field("background"), &a.background);
            write_opt_str(ao.field("initialTheme"), &a.initial_theme);
            ao.finish();
        });
        write_id_name_list(o.field("themes"), self.themes.as_deref(), |t| {
            (&t.id, &t.name)
        });
        write_id_name_list(
            o.field("stateMachines"),
            self.state_machines.as_deref(),
            |s| (&s.id, &s.name),
        );
        o.finish();
        out
    }
}

fn write_id_name_list<T>(
    out: &mut String,
    list: Option<&[T]>,
    fields: impl Fn(&T) -> (&String, &Option<String>),
) {
    match list {
        None => out.push_str("null"),
        Some(items) => write_seq(out, items, |item, out| {
            let (id, name) = fields(item);
            let mut o = ObjWriter::new(out);
            write_str(id, o.field("id"));
            write_opt_str(o.field("name"), name);
            o.finish();
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str = r##"{"version":"2","generator":"g","initial":{"animation":"a1","stateMachine":"sm1"},"animations":[{"id":"a1","name":"First","themes":["t1"],"background":"#fff","initialTheme":"t1"},{"id":"a2","name":null,"themes":null,"background":null,"initialTheme":null}],"themes":[{"id":"t1","name":"Light"}],"stateMachines":[{"id":"sm1","name":null}]}"##;

    #[test]
    fn manifest_round_trips_byte_equal() {
        let m = Manifest::from_json(FULL).expect("parse");
        assert_eq!(m.to_json(), FULL);
    }

    #[test]
    fn manifest_parses_fields() {
        let m = Manifest::from_json(FULL).unwrap();
        assert_eq!(m.version.as_deref(), Some("2"));
        assert_eq!(m.initial.as_ref().unwrap().animation.as_deref(), Some("a1"));
        assert_eq!(m.animations.len(), 2);
        assert_eq!(
            m.animations[0].themes.as_deref(),
            Some(&["t1".to_string()][..])
        );
        assert_eq!(m.themes.as_ref().unwrap()[0].name.as_deref(), Some("Light"));
        assert_eq!(m.state_machines.as_ref().unwrap()[0].id, "sm1");
    }

    #[test]
    fn minimal_manifest_serializes_nulls() {
        let m = Manifest::from_json(r##"{"animations":[{"id":"a"}]}"##).unwrap();
        assert_eq!(
            m.to_json(),
            r##"{"version":null,"generator":null,"initial":null,"animations":[{"id":"a","name":null,"themes":null,"background":null,"initialTheme":null}],"themes":null,"stateMachines":null}"##
        );
    }

    #[test]
    fn missing_animations_is_rejected() {
        assert!(Manifest::from_json(r##"{"version":"2"}"##).is_none());
        assert!(Manifest::from_json("not json").is_none());
    }

    #[test]
    fn animation_without_id_is_rejected() {
        assert!(Manifest::from_json(r##"{"animations":[{"name":"x"}]}"##).is_none());
    }

    #[test]
    fn wrong_typed_initial_is_rejected() {
        assert!(Manifest::from_json(r##"{"initial":"oops","animations":[{"id":"a"}]}"##).is_none());
        assert!(Manifest::from_json(r##"{"initial":null,"animations":[{"id":"a"}]}"##).is_some());
    }
}
