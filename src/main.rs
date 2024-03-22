use mysql_data_export::utils::exit_on_error;

fn main() {
    exit_on_error(mysql_data_export::run());
}
