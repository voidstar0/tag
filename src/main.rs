mod error;

use std::path::PathBuf;
use std::{fs, path::Path};

use clap::Command;
use directories::BaseDirs;
use error::GeneralError;
use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct Location {
    location: String,
}

fn mark_path(connection: Connection, path: &str, tags: &str) -> Result<(), GeneralError> {
    let dir = PathBuf::from(path.trim());
    if !Path::new(&dir).exists() {
        panic!("Path does not exist");
    }

    let absolute_path = fs::canonicalize(&dir)?.to_string_lossy().to_string();

    for tag in tags.split(",") {
        connection.execute(
            "INSERT OR IGNORE INTO tagged (location, tag) VALUES (?1, ?2)",
            &[&absolute_path, &tag.trim().into()],
        )?;
    }
    Ok(())
}

fn find_path(connection: Connection, tags: &str) -> Result<(), rusqlite::Error> {
    for tag in tags.split(",") {
        let mut statement = connection.prepare("SELECT location FROM tagged WHERE tag LIKE ?")?;

        let paths = statement.query_map(&[&tag.trim()], |row| {
            Ok(Location {
                location: row.get(0)?,
            })
        })?;

        for path in paths {
            println!("{}", path?.location);
        }
    }

    Ok(())
}

fn main() -> Result<(), GeneralError> {
    let mut dir = PathBuf::new();
    if let Some(base_dirs) = BaseDirs::new() {
        dir.push(base_dirs.config_dir());
        dir.push("tag");
        dir.set_file_name("tags.db");
    }

    let path = Path::new(&dir);

    if !path.exists() {
        if let Some(parent) = dir.parent() {
            fs::create_dir_all(parent)?;
        }
    }

    let connection = Connection::open(dir)?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS tagged (
             id integer primary key,
             location text not null,
             tag text not null,
             UNIQUE(location, tag)
         );",
        [],
    )?;

    let matches = clap::command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("mark")
                .about("Give a path specified tags")
                .arg(clap::arg!([PATH]))
                .arg(clap::arg!([TAGS])),
        )
        .subcommand(
            Command::new("find")
                .about("Finds a path from tags")
                .arg(clap::arg!([TAGS])),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("mark", sub_matches)) => {
            let path: String = sub_matches.value_of_t_or_exit("PATH");
            let tags: String = sub_matches.value_of_t_or_exit("TAGS");
            mark_path(connection, &path, &tags)?;
        }
        Some(("find", sub_matches)) => {
            let tags: String = sub_matches.value_of_t_or_exit("TAGS");
            find_path(connection, &tags)?;
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    Ok(())
}
