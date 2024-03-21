use crate::traits::*;
use mysql:: Row;
use std::{fmt::Display, fs, io::Write, process};

pub fn table_to_csv(out_dir: &str, file_name: &str, rows: Vec<Row>) {
    if !rows.is_empty() {
        let mut file = exit_on_error(
            fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&format!("{out_dir}/{file_name}.csv")),
        );

        // write column names
        exit_on_error(
            file.write_all(
                rows[0]
                    .columns()
                    .iter()
                    .map(|c| String::from(c.name_str()))
                    .reduce(|a, b| a + "," + &b)
                    .unwrap()
                    .as_bytes(),
            ),
        );
        exit_on_error(file.write(b"\n"));

        // write values
        rows.into_iter().for_each(|row| {
            exit_on_error(
                file.write_all(
                    (0..row.len())
                        .map(|i| row.get_value(i))
                        .reduce(|c1, c2| c1 + "," + &c2)
                        .unwrap_or("".to_string())
                        .as_bytes(),
                ),
            );
            exit_on_error(file.write(b"\n"));
        });
    }
}

pub fn exit_on_error<T, E: Display>(res: Result<T, E>) -> T {
    match res {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}
