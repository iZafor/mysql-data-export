mod traits;
pub mod utils;

use crossterm::{self, cursor, style::Print, terminal, ExecutableCommand};
use mysql::{prelude::Queryable, OptsBuilder};
use std::io::stdout;
use utils::*;

type MError<T> = Result<T, Box<dyn std::error::Error>>;

pub fn run() -> MError<()> {
    terminal::enable_raw_mode()?;

    stdout()
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(terminal::EnableLineWrap)?
        .execute(cursor::MoveTo(0, 0))?;

    // establish connection
    let pass = read_pass()?;
    let mut conn = mysql::Conn::new(
        OptsBuilder::new()
            .ip_or_hostname(Some("127.0.0.1"))
            .user(Some("root"))
            .pass(Some(&pass)),
    )?;

    // select databases
    let dbs = conn.exec::<String, &str, ()>("SHOW DATABASES", ())?;
    if !dbs.is_empty() {
        let dbs = get_selectetions(&dbs, "Select Databases:")?;
        if !dbs.is_empty() {
            let mut dir_buf = String::new();
            stdout()
                .execute(cursor::MoveToNextLine(1))?
                .execute(Print("Export directory: "))?;
            read_to_string(&mut dir_buf, false)?;
            std::fs::create_dir(&dir_buf)?;

            for db in dbs {
                let mut conn = mysql::Conn::new(
                    OptsBuilder::new()
                        .ip_or_hostname(Some("127.0.0.1"))
                        .user(Some("root"))
                        .pass(Some(&pass))
                        .db_name(Some(db)),
                )?;

                // export tables
                let tables = conn.exec::<String, &str, ()>("SHOW TABLES", ())?;
                if !tables.is_empty() {
                    let tables = get_selectetions(&tables, &format!("Select Tables for {db}: "))?;
                    if !tables.is_empty() {
                        let export_dir = format!("{dir_buf}/{db}");
                        std::fs::create_dir(&export_dir)?;
                        export_tables(tables, &mut conn, &export_dir)?;
                    }
                }
            }
        }
    }

    stdout()
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?;
    terminal::disable_raw_mode()?;
    Ok(())
}
