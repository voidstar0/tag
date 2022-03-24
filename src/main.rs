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

fn mark_path(mut connection: Connection, path: &str, tags: &str) -> Result<(), GeneralError> {
    let dir = PathBuf::from(path.trim());
    if !Path::new(&dir).exists() {
        panic!("Path does not exist");
    }

    let absolute_path = fs::canonicalize(&dir)?.to_string_lossy().to_string();

    // Use a transaction in-case we fail to insert a tag at some point.
    let tx = connection.transaction()?;

    for tag in tags.split(',') {
        tx.execute(
            "INSERT OR IGNORE INTO tagged (location, tag) VALUES (?1, ?2)",
            &[&absolute_path, &tag.trim().into()],
        )?;
    }

    tx.commit()?;

    Ok(())
}

fn find_path(connection: Connection, tags: &str, in_cwd: bool) -> Result<(), GeneralError> {
    for tag in tags.split(',') {
        let mut query = String::from("SELECT location FROM tagged WHERE tag LIKE ?");
        let mut params: Vec<String> = vec![tag.trim().into()];

        if in_cwd {
            let cwd = std::env::current_dir().and_then(fs::canonicalize)?;
            let cwd = cwd.to_str().expect("CWD is not a valid utf8 string");

            query.push_str(" AND location LIKE ?");
            params.push(format!("{cwd}%"));
        }

        let mut statement = connection.prepare(&query)?;

        let params = rusqlite::params_from_iter(params);
        let paths = statement.query_map(params, |row| {
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
        .subcommand(Command::new("find").about("Finds a path from tags").args(&[
            clap::arg!(-c --"in-cwd" [IN_CWD] "filters by paths in the current working directory"),
            clap::arg!([TAGS]),
        ]))
        .get_matches();

    match matches.subcommand() {
        Some(("mark", sub_matches)) => {
            let path: String = sub_matches.value_of_t_or_exit("PATH");
            let tags: String = sub_matches.value_of_t_or_exit("TAGS");
            mark_path(connection, &path, &tags)?;
        }
        Some(("find", sub_matches)) => {
            let tags: String = sub_matches.value_of_t_or_exit("TAGS");
            let in_cwd: bool = sub_matches
                .value_of_t("in-cwd")
                .unwrap_or_else(|_| sub_matches.is_present("in-cwd"));
            find_path(connection, &tags, in_cwd)?;
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    };

    Ok(())
}
