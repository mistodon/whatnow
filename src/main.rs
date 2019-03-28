use std::{
    collections::BTreeMap,
    fs::File,
    io::Write,
};
use failure::{Error, err_msg};
use rand::seq::{SliceRandom};
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Dotfile {
    projects: Vec<String>,
    counts: BTreeMap<String, usize>,
}

fn main() -> Result<(), Error> {
    let r = &mut rand::thread_rng();

    let dotfile = option_env!("WHATNOW_DOTFILE_DIR").ok_or_else(|| err_msg("whatnow: WHATNOW_DOTFILE_DIR environment variable not set."))?;
    let dotfile_path = format!("{}/.whatnow.yml", dotfile);
    let mut dotfile: Dotfile = match std::fs::read_to_string(&dotfile_path) {
        Ok(yaml) => serde_yaml::from_str(&yaml)?,
        Err(_) => Dotfile::default(),
    };

    let command = std::env::args().nth(1);
    let command = command.as_ref().map(|s| s.as_ref());

    match command {
        Some("reset") => {
            dotfile.counts.clear();
        }
        Some("path") => {
            println!("{}", dotfile_path);
        }
        None => {
            let project_counts = dotfile.projects.iter().map(|name| {
                let count = dotfile.counts.get(name).unwrap_or(&0);
                (name, *count)
            }).collect::<Vec<_>>();

            let min_count = project_counts.iter().map(|it| it.1).min().unwrap_or(0);
            let mut choices = project_counts.iter().filter(|it| it.1 < min_count + 4).collect::<Vec<_>>();
            choices.shuffle(r);

            let stdin = std::io::stdin();
            let mut answer = String::with_capacity(4);

            for choice in choices {
                print!("'{}' is something you could do? [y/n] ", choice.0);
                std::io::stdout().flush()?;
                stdin.read_line(&mut answer)?;
                if answer.starts_with("y") {
                    let count = dotfile.counts.entry(choice.0.clone()).or_insert(0);
                    *count += 1;
                    break;
                }
            }
        }
        Some(_) => failure::bail!("Unrecognized subcommand."),
    }

    let mut file = File::create(&dotfile_path)?;
    serde_yaml::to_writer(&mut file, &dotfile)?;

    Ok(())
}
