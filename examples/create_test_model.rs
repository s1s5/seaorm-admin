use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, Set};
use seaorm_admin::entity::{author, test_model};

#[tokio::main]
async fn main() {
    env_logger::init();

    let conn = sea_orm::Database::connect(std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Could not connect to database");

    // println!("{:?}", author::Entity::find().count(&conn).await);

    // for _ in 0..100 {
    //     let mut model = author::ActiveModel {
    //         ..Default::default()
    //     };
    //     model.name = Set(format!("{}", uuid::Uuid::new_v4()));
    //     model.insert(&conn).await.expect("insert failed");
    // }

    use sea_orm::ColumnTrait;
    println!("{:?}", test_model::Column::EnumString.as_column_ref());
    println!("string => {:?}", test_model::Column::EnumString.def());
    println!("i32 => {:?}", test_model::Column::EnumI32.def());
    // let mut a = test_model::ActiveModel {
    //     ..Default::default()
    // };
    // let v: sea_orm::Value = sea_orm::Value::String(Some(Box::new("X".to_string())));
    // a.set(test_model::Column::EnumString, v);
    // a.enum_string = v.unwrap();

    // use sea_orm::{ActiveEnum, Iterable};
    // let v: Vec<_> = test_model::Category::iter().collect();
    // println!(
    //     "{:?}, {}",
    //     test_model::Category::Big,
    //     test_model::Category::Big.to_value()
    // );
    // println!(
    //     "{:?}, {}",
    //     test_model::Color::Black,
    //     test_model::Color::Black.to_value()
    // );

    // a.enum_string = Set(Some(test_model::Category::Big));
    // a.enum_i32 = Set(Some(test_model::Color::Black));
    // println!("{:?}", a);

    // let mut a = test_model::ActiveModel {
    //     ..Default::default()
    // };

    // a.char_f = Set(Some("char field".to_string()));
    // a.string_f = Set(Some("string field".to_string()));
    // a.text_f = Set(Some("text field".to_string()));
    // a.tiny_integer_f = Set(Some(1));
    // a.small_integer_f = Set(Some(2));
    // a.integer_f = Set(Some(3));
    // a.big_integer_f = Set(Some(4));
    // a.tiny_unsigned_f = Set(Some(5));
    // a.small_unsigned_f = Set(Some(6));
    // a.unsigned_f = Set(Some(7));
    // a.big_unsigned_f = Set(Some(8));
    // a.float_f = Set(Some(0.1));
    // a.double_f = Set(Some(0.01));
    // a.decimal_f = Set(Some(Decimal::new(33, 2)));
    // a.date_time_f = Set(Some(chrono::Utc::now().naive_utc()));
    // a.timestamp_f = Set(Some(chrono::Utc::now().naive_utc()));
    // a.timestamp_with_time_zone_f = Set(Some(
    //     chrono::Utc::now().with_timezone(&chrono::FixedOffset::east(0)),
    // ));
    // a.time_f = Set(Some(chrono::NaiveTime::from_hms(13, 30, 0)));
    // a.date_f = Set(Some(chrono::NaiveDate::from_ymd(2023, 4, 1)));
    // a.binary_f = Set(Some(vec![1, 2, 3, 4]));
    // // a.var_binary_f = Set(Some(vec![1, 2, 3, 4]));
    // // a.bit_f = Set(Some(vec![1, 2, 3, 4]));
    // // a.var_bit_f = Set(Some(vec![1, 2, 3, 4]));
    // // a.bit_f = Set(Some("01010101".to_string()));
    // // a.var_bit_f = Set(Some("010".to_string()));
    // a.boolean_f = Set(Some(true));
    // a.json_f = Set(Some(serde_json::json!({"a": "b", "c": 3})));
    // a.json_binary_f = Set(Some(serde_json::json!({"d": "e", "f": 8})));
    // a.uuid_f = Set(Some(uuid::Uuid::new_v4()));
    // // a.cidr_f = Set(Some("192.168.1.0/24".to_string()));
    // // a.inet_f = Set(Some("192.168.1.1/24".to_string()));
    // // a.mac_addr_f = Set(Some("01:23:45:67:89:ab".to_string()));
    // a.enum_string = Set(Some(test_model::Category::Big));
    // a.enum_i32 = Set(Some(test_model::Color::Black));

    // let r = a.insert(&conn).await.expect("insert failed");
    // println!("{:?}", r);

    // let e = test_model::Entity::find()
    //     .one(&conn)
    //     .await
    //     .unwrap()
    //     .unwrap();
    // println!("{:?}", e.enum_string);
}
