use sea_orm::{ColumnTrait, ColumnType};

#[derive(Debug, Clone)]
pub struct AdminField {
    pub column_type: ColumnType,
    pub name: String,
    pub editable: bool,
    pub hidden: bool,
    pub required: bool,
    pub help_text: Option<String>,
    pub nullable: bool,
}

impl AdminField {
    pub fn create_from<T>(column: &T, editable: bool) -> Self
    where
        T: ColumnTrait,
    {
        AdminField {
            column_type: column.def().get_column_type().clone(),
            name: column.to_string(),
            editable: editable,
            hidden: false,
            required: false,
            help_text: None,
            nullable: column.def().is_null(),
        }
    }
}
