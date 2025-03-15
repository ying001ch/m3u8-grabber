mod service;
pub mod util;

use m3u8_rs::MediaPlaylist;
use sqlx::{prelude::*, sqlite::SqlitePoolOptions, ColumnIndex, Pool, Sqlite};
use util::decodeHeaders;

use crate::{config::Signal, M3u8Item::M3u8Entity};

pub struct TaskEntity {
    id: u32,
    err_msg: String,
    hash: String, // taskId
    total: usize,
    finished: usize,
    state: Signal,
    file_name: String,
    media_play_list: MediaPlaylist,
    key: [u8; 16],
    iv: [u8; 16],
    key_num: usize,

    headers: Vec<(String, String)>,
    url_prefix: Option<String>,
    save_path: String,
    temp_path: String,
    no_combine: bool,
}

const create_table_sql: &str = r#"
CREATE TABLE if not exists TaskEntity (
    id INTEGER PRIMARY KEY,                     --0 u32
    err_msg TEXT NOT NULL,                      -- String
    hash TEXT NOT NULL,                         -- String (taskId)
    total INTEGER NOT NULL,                     -- usize
    finished INTEGER NOT NULL,                  -- usize
    state INTEGER NOT NULL,                     --5 Signal (假设为整数枚举)
    file_name TEXT NOT NULL,                    -- String
    media_play_list TEXT NOT NULL,              -- MediaPlaylist (假设序列化为 JSON 字符串)
    key BLOB NOT NULL,                          -- [u8; 16]
    iv BLOB NOT NULL,                           -- [u8; 16]
    key_num INTEGER NOT NULL,                   --10 usize
    headers TEXT NOT NULL,                      -- Vec<(String, String)> (假设序列化为 JSON 字符串)
    url_prefix TEXT,                            -- Option<String> (允许 NULL)
    save_path TEXT NOT NULL,                    -- String
    temp_path TEXT NOT NULL,                    -- String
    no_combine INTEGER NOT NULL                 --15 bool (0 或 1)
);
"#;

impl<'r, R> FromRow<'r, R> for M3u8Entity
where
    R: Row,
    usize: ColumnIndex<R>,
    String: Decode<'r, R::Database> + Type<R::Database>,
    i32: Decode<'r, R::Database> + Type<R::Database>,
    bool: Decode<'r, R::Database> + Type<R::Database>,
    Option<String>: Decode<'r, R::Database> + Type<R::Database>,
    &'r [u8]: Decode<'r, R::Database> + Type<R::Database>,
{
    fn from_row(row:  &'r R) -> Result<Self, sqlx::Error> {
        let mut en = M3u8Entity::default();
        
        // en.total = row.try_get::<usize,_>(3)?;
        en.content = row.try_get::<String,_>(7)?;
        en.key.copy_from_slice( row.try_get::<&[u8],_>(8)?);
        en.iv.copy_from_slice( row.try_get::<&[u8],_>(9)?);
        en.key_num = row.try_get::<i32,_>(10)? as usize;

        en.headers = decodeHeaders(row.try_get::<String,_>(11)?);
        en.url_prefix = row.try_get::<Option<String>,_>(12)?;
        en.save_path = row.try_get::<String,_>(13)?;
        en.temp_path = row.try_get::<String,_>(14)?;
        en.no_combine = row.try_get::<bool,_>(15)?;

        Ok(en)
    }
}


pub async fn get_conn() -> Result<Pool<Sqlite>, sqlx::Error>{
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://local.db?mode=rwc")
        .await;
    pool
}

#[tokio::test] // Requires the `attributes` feature of `async-std`
async fn test_create_table() -> Result<(), sqlx::Error> {
    let db = get_conn().await?;

    // 执行 SQL 语句来创建表
    let res = sqlx::query(create_table_sql)
       .execute(&db)
       .await?;
    println!("rows_affected: {}", res.rows_affected());
    Ok(())
}

#[tokio::test] // Requires the `attributes` feature of `async-std`
async fn main() -> Result<(), sqlx::Error> {
    // Create a connection pool
    //  for MySQL/MariaDB, use MySqlPoolOptions::new()
    //  for SQLite, use SqlitePoolOptions::new()
    //  etc.
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://local.db?mode=rwc")
        .await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
    let row: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM sqlite_master WHERE type = 'table';")
            .fetch_all(&pool)
            .await?;
    println!(
        "rows : {:?}",
        row.iter()
            .map(|n| n.0.clone())
            .reduce(|x, y| format!("{},{}", x, y))
    );

    // 使用sqlx更新表post 的 title 字段
    let res = sqlx::query("UPDATE post SET title = ? WHERE id = ?")
        .bind("new title")
        .bind(1)
        .execute(&pool)
        .await?;

    // 使用sqlx查询表post数据
    let rows = sqlx::query_as::<_, (i32, String, String)>("SELECT * FROM post where id=?")
        .bind(1)
        .fetch_all(&pool)
        .await?;
    for row in rows {
        println!("{:?}", row);
    }

    Ok(())
}
