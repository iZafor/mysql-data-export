mod traits;
mod utils;

use mysql::{prelude::*, Row};
use rpassword;
use std::io::Write;

fn main() {
    let pass = rpassword::prompt_password("Password: ").unwrap();
    let mut db_name = String::new();
    print!("Database: ");
    utils::exit_on_error(std::io::stdout().flush());
    utils::exit_on_error(std::io::stdin().read_line(&mut db_name));

    let mut conn = utils::exit_on_error(mysql::Conn::new(
        mysql::OptsBuilder::new()
            .ip_or_hostname(Some("127.0.0.1"))
            .user(Some("root"))
            .pass(Some(pass))
            .db_name(Some(db_name.trim())),
    ));

    // create output directory
    let mut out_dir = String::new();
    print!("Output directory path: ");
    utils::exit_on_error(std::io::stdout().flush());
    utils::exit_on_error(std::io::stdin().read_line(&mut out_dir));
    let out_dir = out_dir.trim();
    utils::exit_on_error(std::fs::create_dir(out_dir));

    for table in utils::exit_on_error(conn.exec::<String, &str, ()>("SHOW TABLES", ())) {
        if let Ok(rows) = conn.exec::<Row, String, ()>(format!("SELECT * FROM {}", &table), ()) {
            utils::table_to_csv(out_dir, &table, rows);
        }
    }
}
