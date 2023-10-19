use anyhow::Result;
use clap::Parser;
use projector_cli::config::{Config, Operation};
use projector_cli::opts::Opts;
use projector_cli::projector::Projector;

fn main() -> Result<()> {
    let config: Config = Opts::parse().try_into()?;
    let mut projector = Projector::from(&config);

    match config.operation {
        Operation::Print(None) => {
            let value = projector.get_value_all();
            let value = serde_json::to_string_pretty(&value)?;
            println!("{}", value);
        }
        Operation::Print(Some(k)) => {
            if let Some(x) = projector.get_value(&k) {
                println!("{}", x)
            }
        }
        Operation::Add(k, v) => {
            projector.set_value(&k, &v);
            projector.save()?;
        }
        Operation::Remove(k) => {
            projector.remove_value(&k);
            projector.save()?;
        }
    }

    return Ok(());
}
