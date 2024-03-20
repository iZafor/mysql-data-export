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
}

impl RowEx for Row {
    fn get_secure<T: FromValue>(&self, i: usize) -> Option<T> {
        self.get_opt(i).and_then(|v| v.ok())
    }
}