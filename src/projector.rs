use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Data {
    pub projector: HashMap<PathBuf, HashMap<String, String>>,
}
pub struct Projector {
    config: Config,
    data: Data,
}
fn default_projector(config: &Config) -> Projector {
    let data = Data::default();
    return Projector {
        config: config.clone(),
        data,
    };
}
impl From<&Config> for Projector {
    fn from(config: &Config) -> Self {
        if std::fs::metadata(&config.config).is_err() {
            return default_projector(config);
        }

        if let Ok(data) = std::fs::read_to_string(&config.config) {
            let data = serde_json::from_str(&data);
            if let Ok(data) = data {
                return Projector {
                    config: config.clone(),
                    data,
                };
            }
        }

        return default_projector(config);
    }
}

impl Projector {
    pub fn get_value_all(&self) -> HashMap<&String, &String> {
        let mut current = Some(self.config.pwd.as_path());
        let mut paths = vec![];
        while let Some(path) = current {
            paths.push(path);
            current = path.parent();
        }

        let mut out = HashMap::new();
        for path in paths.into_iter().rev() {
            if let Some(map) = self.data.projector.get(path) {
                out.extend(map.iter())
            }
        }

        return out;
    }

    pub fn get_value(&self, key: &str) -> Option<&String> {
        let mut current = Some(self.config.pwd.as_path());
        let mut out = None;

        while let Some(path) = current {
            if let Some(dir) = self.data.projector.get(path) {
                let value = dir.get(key);
                if value.is_some() {
                    out = value;
                    break;
                }
            }
            current = path.parent();
        }
        return out;
    }

    pub fn set_value(&mut self, key: &str, value: &str) {
        self.data
            .projector
            .entry(self.config.pwd.clone())
            .or_default()
            .insert(key.into(), value.into());
    }

    pub fn remove_value(&mut self, key: &str) {
        self.data
            .projector
            .get_mut(&self.config.pwd)
            .map(|x| x.remove(key));
    }

    pub fn save(&self) -> Result<()> {
        if let Some(p) = self.config.config.parent() {
            if std::fs::metadata(&p).is_err() {
                std::fs::create_dir_all(p)?;
            }
        }
        let contents = serde_json::to_string_pretty(&self.data)?;
        std::fs::write(&self.config.config, contents)?;
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::{Data, Projector};
    use crate::config::{Config, Operation};
    use collection_macros::hashmap;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn get_data() -> HashMap<PathBuf, HashMap<String, String>> {
        return hashmap! {
            PathBuf::from("/") => hashmap! {
                "foo".into() => "bar1".into(),
                "fem".into() => "meh".into(),
            },
            PathBuf::from("/foo") => hashmap! {
                "foo".into() => "bar2".into(),
            },
            PathBuf::from("/foo/bar") => hashmap! {
                "foo".into() => "bar3".into(),
            },
        };
    }
    fn get_projector(pwd: PathBuf) -> Projector {
        return Projector {
            config: Config {
                pwd,
                config: PathBuf::from(""),
                operation: Operation::Print(None),
            },
            data: Data {
                projector: get_data(),
            },
        };
    }

    #[test]
    fn get_value_all() {
        let projector = get_projector(PathBuf::from("/foo/bar"));
        assert_eq!(
            projector.get_value_all(),
            HashMap::from([
                (&String::from("foo"), &String::from("bar3")),
                (&String::from("fem"), &String::from("meh")),
            ])
        );
    }

    #[test]
    fn get_value() {
        let mut projector = get_projector(PathBuf::from("/foo/bar"));
        assert_eq!(projector.get_value("foo"), Some(&"bar3".to_string()));
        projector = get_projector(PathBuf::from("/foo"));
        assert_eq!(projector.get_value("foo"), Some(&"bar2".to_string()));
        assert_eq!(projector.get_value("fem"), Some(&"meh".to_string()));
    }

    #[test]
    fn set_value() {
        let mut projector = get_projector(PathBuf::from("/foo/bar"));
        projector.set_value("foo", "bar4");
        assert_eq!(projector.get_value("foo"), Some(&"bar4".to_string()));

        projector.set_value("fem", "meeeh");
        assert_eq!(projector.get_value("fem"), Some(&"meeeh".to_string()));

        projector = get_projector(PathBuf::from("/"));
        assert_eq!(projector.get_value("fem"), Some(&"meh".to_string()));
    }

    #[test]
    fn remove_value() {
        let mut projector = get_projector(PathBuf::from("/foo/bar"));
        projector.remove_value("fem");
        assert_eq!(projector.get_value("fem"), Some(&"meh".to_string()));

        projector.remove_value("foo");
        assert_eq!(projector.get_value("foo"), Some(&"bar2".to_string()));
    }
}
