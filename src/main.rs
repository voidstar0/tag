use std::{fs, path::Path};
use std::path::PathBuf;
use clap::{Command};
use directories::BaseDirs;
use rusqlite::{Connection};

#[derive(Debug)]
struct Location {
    location: String,
}

fn mark_path(connection: Connection, args: (Option<&str>, Option<&str>)) -> Result<(), Box<dyn std::error::Error>> {
    match args {
        (Some(path), Some(tags)) => {
            let dir = PathBuf::from(path.trim());
            if !Path::new(&dir).exists() {
                panic!("Path does not exist");
            }

            let absolute_path = fs::canonicalize(&dir).unwrap().to_string_lossy().to_string();
            for tag in tags.split(",") {
                connection
                    .execute("INSERT OR IGNORE INTO tagged (location, tag) VALUES (?1, ?2)", &[&absolute_path, &tag.trim().into()])
                    .unwrap();
            }
            Ok(())
        },
        _ => panic!("Bruh moment")
    }
}

fn find_path(connection: Connection, tags: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    match tags {
        Some(tags) => {
            for tag in tags.split(",") {
                let mut statement = connection
                    .prepare("SELECT location FROM tagged WHERE tag LIKE ?")?;

                let paths = statement.query_map(&[&tag.trim()], |row| {
                    Ok(Location { location: row.get(0)? })
                })?;

                for path in paths {
                    println!("{}", path.unwrap().location);
                }
            }
            Ok(())
        },
        _ => panic!("Bruh moment")
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dir = PathBuf::new();
    if let Some(base_dirs) = BaseDirs::new() {
        dir.push(base_dirs.config_dir());
        dir.push("tag");
        dir.set_file_name("tags.db");
    }

    let path = Path::new(&dir);
    if !path.exists() {
        fs::create_dir_all(&dir.parent().unwrap()).unwrap();
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

    return match matches.subcommand() {
        Some(("mark", sub_matches)) => mark_path(connection, (
            sub_matches.value_of("PATH"),
            sub_matches.value_of("TAGS"))
        ),
        Some(("find", sub_matches)) => find_path(connection, sub_matches.value_of("TAGS")),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    };
}
