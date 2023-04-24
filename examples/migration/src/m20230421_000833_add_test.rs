use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TestModel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TestModel::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TestModel::CharF).char().char_len(32))
                    .col(ColumnDef::new(TestModel::StringF).string())
                    .col(ColumnDef::new(TestModel::TextF).string())
                    .col(ColumnDef::new(TestModel::TinyIntegerF).tiny_integer())
                    .col(ColumnDef::new(TestModel::SmallIntegerF).small_integer())
                    .col(ColumnDef::new(TestModel::IntegerF).integer())
                    .col(ColumnDef::new(TestModel::BigIntegerF).big_integer())
                    .col(ColumnDef::new(TestModel::TinyUnsignedF).tiny_unsigned())
                    .col(ColumnDef::new(TestModel::SmallUnsignedF).small_unsigned())
                    .col(ColumnDef::new(TestModel::UnsignedF).unsigned())
                    .col(ColumnDef::new(TestModel::BigUnsignedF).big_unsigned())
                    .col(ColumnDef::new(TestModel::FloatF).float())
                    .col(ColumnDef::new(TestModel::DoubleF).double())
                    .col(
                        ColumnDef::new(TestModel::DecimalF)
                            .decimal()
                            .decimal_len(32, 2),
                    )
                    .col(ColumnDef::new(TestModel::DateTimeF).date_time())
                    .col(ColumnDef::new(TestModel::TimestampF).timestamp())
                    .col(
                        ColumnDef::new(TestModel::TimestampWithTimeZoneF)
                            .timestamp_with_time_zone(),
                    )
                    .col(ColumnDef::new(TestModel::TimeF).time())
                    .col(ColumnDef::new(TestModel::DateF).date())
                    // .col(ColumnDef::new(TestModel::YearF).year(Some(MySqlYear::Four)))
                    // .col(
                    //     ColumnDef::new(TestModel::IntervalF)
                    //         .interval(Some(PgInterval::YearToMonth), None),
                    // )
                    .col(ColumnDef::new(TestModel::BinaryF).binary().binary_len(4))
                    .col(ColumnDef::new(TestModel::VarBinaryF).var_binary(8))
                    .col(ColumnDef::new(TestModel::BitF).bit(Some(8)))
                    .col(ColumnDef::new(TestModel::VarBitF).bit(None))
                    .col(ColumnDef::new(TestModel::BooleanF).boolean())
                    // .col(ColumnDef::new(TestModel::MoneyF).money().money_len(8, 2))
                    .col(ColumnDef::new(TestModel::JsonF).json())
                    .col(ColumnDef::new(TestModel::JsonBinaryF).json_binary())
                    .col(ColumnDef::new(TestModel::UuidF).uuid())
                    // .col(ColumnDef::new(TestModel::ArrayF).array(ColumnType::Integer))
                    .col(ColumnDef::new(TestModel::CidrF).cidr())
                    .col(ColumnDef::new(TestModel::InetF).inet())
                    .col(ColumnDef::new(TestModel::MacAddrF).mac_address())
                    .col(ColumnDef::new(TestModel::EnumString).string())
                    .col(ColumnDef::new(TestModel::EnumI32).integer())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TestModel::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum TestModel {
    Table,
    Id,
    CharF,
    StringF,
    TextF,
    TinyIntegerF,
    SmallIntegerF,
    IntegerF,
    BigIntegerF,
    TinyUnsignedF,
    SmallUnsignedF,
    UnsignedF,
    BigUnsignedF,
    FloatF,
    DoubleF,
    DecimalF,
    DateTimeF,
    TimestampF,
    TimestampWithTimeZoneF,
    TimeF,
    DateF,
    // YearF,
    // IntervalF,
    BinaryF,
    VarBinaryF,
    BitF,
    VarBitF,
    BooleanF,
    // MoneyF,
    JsonF,
    JsonBinaryF,
    UuidF,
    // ArrayF,
    CidrF,
    InetF,
    MacAddrF,
    EnumString,
    EnumI32,
}
