use failure::{err_msg, Error};
use rand::seq::SliceRandom;
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, io::Write};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Dotfile {
    projects: Vec<Project>,
    counts: BTreeMap<String, usize>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Project {
    name: String,
    at: Vec<String>,
}

fn main() -> Result<(), Error> {
    let r = &mut rand::thread_rng();

    let local_exists = std::path::Path::new(".whatnow.yml").exists();
    let dotfile_path = option_env!("WHATNOW_DOTFILE_DIR")
        .ok_or_else(|| err_msg("whatnow: WHATNOW_DOTFILE_DIR environment variable not set."))?;
    let dotfile_path = if local_exists {
        ".whatnow.yml".into()
    } else {
        format!("{}/.whatnow.yml", dotfile_path)
    };
    let mut dotfile: Dotfile = match std::fs::read_to_string(&dotfile_path) {
        Ok(yaml) => serde_yaml::from_str(&yaml)?,
        Err(_) => Dotfile::default(),
    };

    let args = std::env::args().collect::<Vec<_>>();
    let command = args.get(1).map(String::as_ref);
    let command_arg = args.get(2);

    let project_counts = dotfile
        .projects
        .iter()
        .map(|project| {
            let count = dotfile.counts.get(&project.name).unwrap_or(&0);
            (project, *count)
        })
        .collect::<Vec<_>>();

    match command {
        Some("reset") => {
            dotfile.counts.clear();
        }
        Some("path") => {
            println!("{}", dotfile_path);
        }
        Some("inc") => {
            println!("Increment which count?");
            for (index, project) in project_counts.iter().enumerate() {
                println!("{: >2}) {}", index, project.0.name);
            }

            let mut answer = String::with_capacity(4);
            let stdin = std::io::stdin();
            stdin.read_line(&mut answer)?;

            let index: usize = answer.trim().parse()?;
            let project = project_counts[index].0.name.clone();
            let count = dotfile.counts.entry(project).or_insert(0);
            *count += 1;
        }
        Some("at") if command_arg.is_none() => {
            let mut places = vec![];
            for (project, _) in project_counts {
                places.extend_from_slice(&project.at);
            }
            places.sort();
            places.dedup();
            for place in places {
                println!(" - {}", place);
            }
        }
        Some("at") | None => {
            let min_count = project_counts.iter().map(|it| it.1).min().unwrap_or(0);
            let mut choices = project_counts
                .iter()
                .filter(|it| match command_arg {
                    Some(filter) => it.0.at.contains(filter),
                    None => true,
                })
                .filter(|it| it.1 < min_count + 4)
                .collect::<Vec<_>>();
            choices.shuffle(r);

            let stdin = std::io::stdin();
            let mut answer = String::with_capacity(4);

            for choice in choices {
                print!("'{}' is something you could do? [y/n] ", choice.0.name);
                std::io::stdout().flush()?;
                answer.clear();
                stdin.read_line(&mut answer)?;
                if answer.starts_with("y") {
                    let count = dotfile.counts.entry(choice.0.name.clone()).or_insert(0);
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
