mod traits;
pub mod utils;

use crossterm::{self, cursor, terminal, ExecutableCommand};
use mysql::{prelude::Queryable, OptsBuilder};
use std::io::stdout;
use utils::*;

type MError<T> = Result<T, Box<dyn std::error::Error>>;

pub fn run() -> MError<()> {
    let mut stdout = stdout().lock();
    terminal::enable_raw_mode()?;
    stdout
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?;

    // establish connection
    let mut conn = mysql::Conn::new(
        OptsBuilder::new()
            .ip_or_hostname(Some("127.0.0.1"))
            .user(Some("root"))
            .pass(Some(read_pass(&mut stdout)?)),
    )?;
    conn.select_db(&read_db_name(&mut stdout)?)?;

    // export tables
    let tables = conn.exec::<String, &str, ()>("SHOW TABLES", ())?;
    if !tables.is_empty() {
        let selects = get_selected_tables(&mut stdout, &tables)?;
        let tables = (selects[0] == 0)
            .then(|| {
                selects
                    .iter()
                    .skip(1)
                    .enumerate()
                    .filter_map(|(idx, val)| (*val == 1).then_some(&tables[idx]))
                    .collect::<Vec<_>>()
            })
            .unwrap_or(tables.iter().map(|t| t).collect::<Vec<_>>());

        if !tables.is_empty() {
            export_tables(&mut stdout, tables, &mut conn)?;
        }
    }

    stdout
        .execute(terminal::Clear(terminal::ClearType::All))?
        .execute(cursor::MoveTo(0, 0))?;
    terminal::disable_raw_mode()?;
    Ok(())
}
