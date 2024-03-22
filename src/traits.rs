use mysql::{consts::ColumnType, prelude::FromValue, Row};

pub trait ColumnTypeEx {
    fn is_date_time_type(&self) -> bool;

    fn is_boolean_type(&self) -> bool;
}

impl ColumnTypeEx for ColumnType {
    fn is_date_time_type(&self) -> bool {
        matches!(
            self,
            ColumnType::MYSQL_TYPE_DATE
                | ColumnType::MYSQL_TYPE_DATETIME
                | ColumnType::MYSQL_TYPE_DATETIME2
        )
    }

    fn is_boolean_type(&self) -> bool {
        matches!(self, ColumnType::MYSQL_TYPE_TINY)
    }
}

pub trait RowEx {
    fn get_secure<T: FromValue>(&self, i: usize) -> Option<T>;

    fn get_value(&self, i: usize) -> String;
}

impl RowEx for Row {
    fn get_secure<T: FromValue>(&self, i: usize) -> Option<T> {
        self.get_opt(i).and_then(|v| v.ok())
    }

    fn get_value(&self, i: usize) -> String {
        if self.is_empty() {
            return String::new();
        }

        let data_type = self.columns().get(i).unwrap().column_type();
        if data_type.is_boolean_type() {
            return match self.get_secure::<u8>(i) {
                Some(v) => format!("{v}"),
                None => String::new(),
            };
        } else if data_type.is_character_type() {
            return match self.get_secure::<String>(i) {
                Some(v) => format!("\"{v}\""),
                None => format!("\"\""),
            };
        } else if data_type.is_numeric_type() {
            return match self.get_secure::<f32>(i) {
                Some(v) => format!("{:.2}", v),
                None => String::new(),
            };
        } else if data_type.is_date_time_type() {
            return match self.get_secure::<time::PrimitiveDateTime>(i) {
                Some(v) => format!("\"{v}\""),
                None => format!("\"\""),
            };
        } else {
            return String::new();
        }
    }
}
