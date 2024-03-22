use crate::{traits::*, MError};
use crossterm::{
    cursor,
    event::{read as ev_read, Event, KeyCode, KeyEvent},
    style::Print,
    ExecutableCommand,
};
use mysql::{prelude::*, Row};
use std::{
    fmt::Display,
    fs,
    io::{StdoutLock, Write},
    process,
};

pub fn read_pass(stdout: &mut StdoutLock) -> MError<String> {
    let mut pass_buf = String::new();
    stdout.execute(Print("Enter password: "))?;
    read_to_string(stdout, &mut pass_buf, true)?;
    Ok(pass_buf)
}

pub fn read_db_name(stdout: &mut StdoutLock) -> MError<String> {
    let mut name_buf = String::new();
    stdout.execute(Print("Enter database name: "))?;
    read_to_string(stdout, &mut name_buf, false)?;
    Ok(name_buf)
}

pub fn get_selected_tables(stdout: &mut StdoutLock, tables: &Vec<String>) -> MError<Vec<usize>> {
    stdout
        .execute(Print("Select Tables: "))?
        .execute(cursor::MoveToNextLine(1))?;

    let mut selects: Vec<usize> = vec![0; tables.len() + 1];
    let (_, st_row) = cursor::position()?;
    stdout
        .execute(Print("[?] All"))?
        .execute(cursor::MoveToNextLine(1))?;
    for table in tables.iter() {
        stdout
            .execute(Print(format!("[ ] {table}")))?
            .execute(cursor::MoveToNextLine(1))?;
    }
    stdout.execute(cursor::MoveTo(1, st_row))?;

    loop {
        match ev_read()? {
            Event::Key(ev) => {
                let KeyEvent { code, .. } = ev;
                let (_, curr_row) = cursor::position()?;

                match code {
                    KeyCode::Up => {
                        let new_row = curr_row - 1;
                        if new_row >= st_row {
                            if selects[(curr_row - st_row) as usize] == 0 {
                                stdout.execute(Print(' '))?;
                            }
                            stdout.execute(cursor::MoveTo(1, new_row))?;
                            if selects[(new_row - st_row) as usize] == 0 {
                                stdout.execute(Print('?'))?.execute(cursor::MoveLeft(1))?;
                            }
                        }
                    }
                    KeyCode::Down => {
                        let new_row = curr_row + 1;
                        if new_row <= st_row + tables.len() as u16 {
                            if selects[(curr_row - st_row) as usize] == 0 {
                                stdout.execute(Print(' '))?;
                            }
                            stdout.execute(cursor::MoveTo(1, new_row))?;
                            if selects[(new_row - st_row) as usize] == 0 {
                                stdout.execute(Print('?'))?.execute(cursor::MoveLeft(1))?;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        let idx = (curr_row - st_row) as usize;
                        selects[idx] ^= 1;
                        if selects[idx] == 0 {
                            stdout.execute(Print('?'))?.execute(cursor::MoveLeft(1))?;
                        } else {
                            stdout.execute(Print('x'))?.execute(cursor::MoveLeft(1))?;
                        }
                    }
                    _ => break,
                }
            }
            _ => break,
        }
    }
    stdout
        .execute(cursor::MoveTo(0, st_row))?
        .execute(cursor::MoveToNextLine(st_row + selects.len() as u16))?;
    Ok(selects)
}

pub fn export_tables(
    stdout: &mut StdoutLock,
    tables: Vec<&String>,
    conn: &mut mysql::Conn,
) -> MError<()> {
    let mut dir_buf = String::new();
    stdout.execute(Print("Export directory: "))?;
    read_to_string(stdout, &mut dir_buf, false)?;
    std::fs::create_dir(&dir_buf)?;

    for table in tables.into_iter() {
        table_to_csv(
            &dir_buf,
            table,
            conn.exec::<mysql::Row, String, ()>(format!("SELECT * FROM {table}"), ())?,
        )?;
        stdout
            .execute(Print(format!("{table} is saved on {dir_buf}/{table}.csv")))?
            .execute(cursor::MoveToNextLine(1))?;
    }

    Ok(())
}

pub fn read_to_string(stdout: &mut StdoutLock, buf: &mut String, obscure: bool) -> MError<()> {
    loop {
        match ev_read()? {
            Event::Key(ev) => {
                let KeyEvent { code, .. } = ev;
                match code {
                    KeyCode::Char(c) => {
                        buf.push(c);
                        if obscure {
                            stdout.execute(Print('*'))?;
                        } else {
                            stdout.execute(Print(c))?;
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(_) = buf.pop() {
                            stdout
                                .execute(cursor::MoveLeft(1))?
                                .execute(Print(' '))?
                                .execute(cursor::MoveLeft(1))?;
                        }
                    }
                    _ => break,
                }
            }
            Event::Paste(text) => buf.push_str(&text),
            _ => break,
        }
    }
    stdout.execute(cursor::MoveToNextLine(1))?;
    Ok(())
}

pub fn table_to_csv(out_dir: &str, file_name: &str, rows: Vec<Row>) -> MError<()> {
    if !rows.is_empty() {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&format!("{out_dir}/{file_name}.csv"))?;

        // write column names
        file.write_all(
            rows[0]
                .columns()
                .iter()
                .map(|c| String::from(c.name_str()))
                .reduce(|a, b| a + "," + &b)
                .unwrap()
                .as_bytes(),
        )?;
        file.write(b"\n")?;

        // write values
        for row in rows {
            file.write_all(
                (0..row.len())
                    .map(|i| row.get_value(i))
                    .reduce(|c1, c2| c1 + "," + &c2)
                    .unwrap_or("".to_string())
                    .as_bytes(),
            )?;
            file.write(b"\n")?;
        }
    }

    Ok(())
}

pub fn exit_on_error<T, E: Display>(res: Result<T, E>) -> T {
    match res {
        Ok(v) => v,
        Err(e) => {
            crossterm::terminal::disable_raw_mode().unwrap();
            eprintln!("{e}");
            process::exit(1);
        }
    }
}
