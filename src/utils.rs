use crate::traits::*;
use mysql:: Row;
use std::{fmt::Display, fs, io::Write, process};
use time;

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

        let types = rows[0]
            .columns()
            .iter()
            .map(|c| c.column_type())
            .collect::<Vec<_>>();

        // write values
        rows.into_iter().for_each(|row| {
            exit_on_error(
                file.write_all(
                    (0..row.len())
                        .map(|i| {
                            let data_type = types[i];
                            if data_type.is_boolean_type() {
                                return match row.get_secure::<u8>(i) {
                                    Some(v) => format!("{v}"),
                                    None => String::new(),
                                };
                            } else if data_type.is_character_type() {
                                return match row.get_secure::<String>(i) {
                                    Some(v) => format!("\"{v}\""),
                                    None => format!("\"\""),
                                };
                            } else if data_type.is_numeric_type() {
                                return match row.get_secure::<f32>(i) {
                                    Some(v) => format!("{:.2}", v),
                                    None => String::new(),
                                };
                            } else if data_type.is_date_time_type() {
                                return match row.get_secure::<time::PrimitiveDateTime>(i) {
                                    Some(v) => format!("\"{v}\""),
                                    None => format!("\"\""),
                                };
                            } else {
                                return String::new();
                            }
                        })
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
