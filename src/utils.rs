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
    io::{stdout, Write},
    process,
};

pub fn read_pass() -> MError<String> {
    let mut pass_buf = String::new();
    stdout().execute(Print("Enter password: "))?;
    read_to_string(&mut pass_buf, true)?;
    Ok(pass_buf)
}

pub fn read_db_name() -> MError<String> {
    let mut name_buf = String::new();
    stdout().execute(Print("Enter database name: "))?;
    read_to_string(&mut name_buf, false)?;
    Ok(name_buf)
}

pub fn get_selectetions<'a>(
    options: &'a Vec<String>,
    intro_message: &str,
) -> MError<Vec<&'a String>> {
    stdout()
        .execute(cursor::MoveToNextLine(1))?
        .execute(Print(
            "* Press Enter to toggle selection, any other key to exit",
        ))?
        .execute(cursor::MoveToNextLine(1))?
        .execute(Print(intro_message))?
        .execute(cursor::MoveToNextLine(1))?;
    let selectes = select_from_options(options)?;
    Ok((selectes[0] == 0)
        .then(|| {
            selectes
                .into_iter()
                .skip(1)
                .enumerate()
                .filter_map(|(idx, val)| (val == 1).then_some(&options[idx]))
                .collect::<Vec<_>>()
        })
        .unwrap_or(options.iter().map(|opt| opt).collect::<Vec<_>>()))
}

pub fn select_from_options(options: &Vec<String>) -> MError<Vec<usize>> {
    let mut selects: Vec<usize> = vec![0; options.len() + 1];
    let (_, st_row) = cursor::position()?;
    stdout()
        .execute(Print("[?] All"))?
        .execute(cursor::MoveToNextLine(1))?;
    for opt in options.iter() {
        stdout()
            .execute(Print(format!("[ ] {opt}")))?
            .execute(cursor::MoveToNextLine(1))?;
    }
    stdout().execute(cursor::MoveTo(1, st_row))?;

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
                                stdout().execute(Print(' '))?;
                            }
                            stdout().execute(cursor::MoveTo(1, new_row))?;
                            if selects[(new_row - st_row) as usize] == 0 {
                                stdout().execute(Print('?'))?.execute(cursor::MoveLeft(1))?;
                            }
                        }
                    }
                    KeyCode::Down => {
                        let new_row = curr_row + 1;
                        if new_row <= st_row + options.len() as u16 {
                            if selects[(curr_row - st_row) as usize] == 0 {
                                stdout().execute(Print(' '))?;
                            }
                            stdout().execute(cursor::MoveTo(1, new_row))?;
                            if selects[(new_row - st_row) as usize] == 0 {
                                stdout().execute(Print('?'))?.execute(cursor::MoveLeft(1))?;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        let idx = (curr_row - st_row) as usize;
                        selects[idx] ^= 1;
                        if selects[idx] == 0 {
                            stdout().execute(Print('?'))?.execute(cursor::MoveLeft(1))?;
                        } else {
                            stdout().execute(Print('+'))?.execute(cursor::MoveLeft(1))?;
                        }
                    }
                    _ => break,
                }
            }
            _ => break,
        }
    }
    stdout().execute(cursor::MoveTo(0, st_row + selects.len() as u16))?;
    Ok(selects)
}

pub fn export_tables(tables: Vec<&String>, conn: &mut mysql::Conn, export_dir: &str) -> MError<()> {
    for table in tables.into_iter() {
        table_to_csv(
            export_dir,
            table,
            conn.exec::<mysql::Row, String, ()>(format!("SELECT * FROM {table}"), ())?,
        )?;
        stdout()
            .execute(Print(format!(
                "{table} is saved on {export_dir}/{table}.csv"
            )))?
            .execute(cursor::MoveToNextLine(1))?;
    }
    Ok(())
}

pub fn read_to_string(buf: &mut String, obscure: bool) -> MError<()> {
    let (buf_st_col, buf_st_row) = cursor::position()?;
    loop {
        let (curr_col, _) = cursor::position()?;
        match ev_read()? {
            Event::Key(ev) => {
                let KeyEvent { code, .. } = ev;
                match code {
                    KeyCode::Char(c) => {
                        let idx = (curr_col - buf_st_col) as usize;
                        if idx < buf.len() {
                            buf.insert(idx, c);
                        } else {
                            buf.push(c);
                        }

                        for c in buf.chars().skip(idx) {
                            if obscure {
                                stdout().execute(Print('*'))?;
                            } else {
                                stdout().execute(Print(c))?;
                            }
                        }
                        stdout().execute(cursor::MoveTo(curr_col + 1, buf_st_row))?;
                    }
                    KeyCode::Backspace => {
                        let idx = (curr_col - buf_st_col) as usize;
                        if idx > 0 && idx <= buf.len() {
                            buf.remove(idx - 1);
                            stdout().execute(cursor::MoveLeft(1))?;

                            for c in buf.chars().skip(idx - 1) {
                                if obscure {
                                    stdout().execute(Print('*'))?;
                                } else {
                                    stdout().execute(Print(c))?;
                                }
                            }
                            stdout()
                                .execute(Print(' '))?
                                .execute(cursor::MoveTo(curr_col - 1, buf_st_row))?;
                        }
                    }
                    KeyCode::Left => {
                        if curr_col - 1 >= buf_st_col {
                            stdout().execute(cursor::MoveLeft(1))?;
                        }
                    }
                    KeyCode::Right => {
                        if curr_col + 1 <= buf_st_col + buf.len() as u16 {
                            stdout().execute(cursor::MoveRight(1))?;
                        }
                    }
                    _ => break,
                }
            }
            Event::Paste(text) => buf.push_str(&text),
            _ => break,
        }
    }
    stdout().execute(cursor::MoveToNextLine(1))?;
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
