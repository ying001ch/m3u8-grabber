pub mod service;
pub mod util;

use std::{sync::LazyLock, thread};

use m3u8_rs::MediaPlaylist;
use sqlx::{prelude::*, sqlite::SqlitePoolOptions, ColumnIndex, Pool, Sqlite};
use sqlx_sqlite::SqliteRow;
use util::decode_headers;

use crate::{async_runtime, config::{Signal, TaskState}, M3u8Item::M3u8Entity};

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

impl<'r> FromRow<'r, SqliteRow> for TaskState
{
    fn from_row(row:  &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let mut en: M3u8Entity = M3u8Entity::default();
        
        // en.total = row.try_get::<usize,_>(3)?;
        en.content = row.try_get::<String,_>("media_play_list")?;
        en.key.copy_from_slice( row.try_get::<&[u8],_>("key")?);
        en.iv.copy_from_slice( row.try_get::<&[u8],_>("iv")?);
        en.key_num = row.try_get::<i32,_>("key_num")? as usize;

        en.headers = decode_headers(row.try_get::<String,_>("headers")?);
        en.url_prefix = row.try_get::<Option<String>,_>("url_prefix")?;
        en.save_path = row.try_get::<String,_>("save_path")?;
        en.temp_path = row.try_get::<String,_>("temp_path")?;
        en.no_combine = row.try_get::<bool,_>("no_combine")?;

        let mut task = TaskState::from(&en);
        task.set_ext(
            row.try_get::<u32,_>("finished")? as usize,
            (row.try_get::<u32,_>("state")? as usize).into(),
            row.try_get::<String,_>("err_msg")?.as_str(),
        );
        
        Ok(task)
    }
}
static POOL: LazyLock<Pool<Sqlite>> = LazyLock::new(||{
    thread::spawn(||{
        async_runtime::block_on(async {
            let db = SqlitePoolOptions::new()
                .max_connections(5)
                .connect("sqlite://local.db?mode=rwc")
                .await.unwrap();
            init_table(&db).await.unwrap();
            db
        })
    }).join().unwrap()
});
pub fn get_conn() -> &'static Pool<Sqlite>{
    &POOL
}
pub async fn init_table(db: &Pool<Sqlite>) -> Result<(), sqlx::Error>{
    // 执行 SQL 语句来创建表
    sqlx::query(create_table_sql)
       .execute(db)
       .await?;
    Ok(())
}

#[tokio::test] // Requires the `attributes` feature of `async-std`
async fn test_create_table() -> Result<(), sqlx::Error> {
    let db = get_conn();

    // 执行 SQL 语句来创建表
    let res = sqlx::query(create_table_sql)
       .execute(db)
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
