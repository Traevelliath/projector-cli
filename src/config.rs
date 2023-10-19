use crate::opts::Opts;
use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub operation: Operation,
    pub config: PathBuf,
    pub pwd: PathBuf,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Print(Option<String>),
    Add(String, String),
    Remove(String),
}

impl TryFrom<Opts> for Config {
    type Error = anyhow::Error;

    fn try_from(value: Opts) -> Result<Self> {
        return Ok(Config {
            operation: value.args.try_into()?,
            config: get_config(value.config)?,
            pwd: get_pwd(value.pwd)?,
        });
    }
}

impl TryFrom<Vec<String>> for Operation {
    type Error = anyhow::Error;

    fn try_from(value: Vec<String>) -> Result<Self> {
        let mut value = value;
        if value.is_empty() {
            return Ok(Operation::Print(None));
        }

        let term = value.get(0).expect("Expected to exist");
        if term == "add" {
            if value.len() != 3 {
                return Err(anyhow!(
                    "Operation 'add' expects 2 arguments, got {} instead",
                    value.len() - 1
                ));
            }
            let mut drain = value.drain(1..=2);
            return Ok(Operation::Add(
                drain.next().expect("Expected to exist"),
                drain.next().expect("Expected to exist"),
            ));
        }
        if term == "rm" {
            if value.len() != 2 {
                return Err(anyhow!(
                    "Operation 'remove' expects 1 arguments, got {} instead",
                    value.len() - 1
                ));
            }
            let arg = value.pop().expect("Expected to exist");
            return Ok(Operation::Remove(arg));
        }
        if value.len() > 1 {
            return Err(anyhow!(
                "Operation 'print' expects 0 or 1 arguments, got {} instead",
                value.len()
            ));
        }

        let arg = value.pop().expect("Expected to exist");
        return Ok(Operation::Print(Some(arg)));
    }
}
fn get_config(config: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(value) = config {
        return Ok(value);
    }

    let loc = std::env::var("HOME").context("Unable to find HOME")?;
    let mut loc = PathBuf::from(loc);

    loc.push("projector");
    loc.push("projector.json");

    return Ok(loc);
}
fn get_pwd(pwd: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(value) = pwd {
        return Ok(value);
    }

    let pwd = std::env::current_dir().context("Unable to get current_dir")?;
    return Ok(pwd);
}

#[cfg(test)]
mod test {
    use super::Config;
    use crate::config::Operation;
    use crate::opts::Opts;
    use anyhow::Result;

    #[test]
    fn print_all() -> Result<()> {
        let opts: Config = Opts {
            args: vec![],
            pwd: None,
            config: None,
        }
        .try_into()?;

        assert_eq!(opts.operation, Operation::Print(None));
        return Ok(());
    }
    #[test]
    fn print_key() -> Result<()> {
        let opts: Config = Opts {
            args: vec!["foo".to_string()],
            pwd: None,
            config: None,
        }
        .try_into()?;

        assert_eq!(opts.operation, Operation::Print(Some("foo".to_string())));
        return Ok(());
    }
    #[test]
    fn add_key_value() -> Result<()> {
        let opts: Config = Opts {
            args: vec!["add".to_string(), "foo".to_string(), "bar".to_string()],
            pwd: None,
            config: None,
        }
        .try_into()?;

        assert_eq!(
            opts.operation,
            Operation::Add("foo".to_string(), "bar".to_string())
        );
        return Ok(());
    }
    #[test]
    fn remove_key_value() -> Result<()> {
        let opts: Config = Opts {
            args: vec!["rm".to_string(), "foo".to_string()],
            pwd: None,
            config: None,
        }
        .try_into()?;

        assert_eq!(opts.operation, Operation::Remove("foo".to_string()));
        return Ok(());
    }
}
