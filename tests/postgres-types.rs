use sqlx::{PgConnection, Connect, Row};
use sqlx::postgres::PgRow;

async fn connect() -> anyhow::Result<PgConnection> {
    Ok(PgConnection::connect(dotenv::var("DATABASE_URL")?).await?)
}

macro_rules! test {
    ($name:ident: $ty:ty: $($text:literal == $value:expr),+) => {
        #[cfg_attr(feature = "runtime-async-std", async_std::test)]
        #[cfg_attr(feature = "runtime-tokio", tokio::test)]
        async fn $name () -> anyhow::Result<()> {
            let mut conn = connect().await?;

            $(
                let rec: (bool, $ty) = sqlx::query(&format!("SELECT {} = $1, $1 as _1", $text))
                    .bind($value)
                    .map(|row: PgRow| Ok((row.get(0)?, row.get(1)?)))
                    .fetch_one(&mut conn)
                    .await?;

                assert!(rec.0);
                assert!($value == rec.1);
            )+

            Ok(())
        }
    }
}

test!(postgres_bool: bool: "false::boolean" == false, "true::boolean" == true);

test!(postgres_smallint: i16: "821::smallint" == 821_i16);
test!(postgres_int: i32: "94101::int" == 94101_i32);
test!(postgres_bigint: i64: "9358295312::bigint" == 9358295312_i64);

test!(postgres_real: f32: "9419.122::real" == 9419.122_f32);
test!(postgres_double: f64: "939399419.1225182::double precision" == 939399419.1225182_f64);

test!(postgres_text: String: "'this is foo'" == "this is foo", "''" == "");

#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
async fn postgres_bytes() -> anyhow::Result<()> {
    let mut conn = connect().await?;

    let value = b"Hello, World";

    let rec: (bool, Vec<u8>) = sqlx::query("SELECT E'\\\\x48656c6c6f2c20576f726c64' = $1, $1")
        .bind(&value[..])
        .map(|row: PgRow| Ok((row.get(0)?, row.get(1)?)))
        .fetch_one(&mut conn)
        .await?;

    assert!(rec.0);
    assert_eq!(&value[..], &*rec.1);

    Ok(())
}
