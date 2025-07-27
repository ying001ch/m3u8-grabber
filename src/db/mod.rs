pub mod service;
pub mod util;

use std::{sync::LazyLock, thread};

use db_derive::DbInsertable;
use sqlx::{prelude::*, sqlite::SqlitePoolOptions, Pool, Sqlite};
use sqlx_sqlite::SqliteArguments;
use util::decode_headers;

use crate::{async_runtime, config::{Signal, TaskState}, db::util::encode_headers, M3u8Item::M3u8Entity};

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
const CREATE_TABLE_SQL: &str = r#"
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

pub trait DbInsertable { 
    fn insert_statement() -> String;

    fn as_arguments(&self) -> SqliteArguments<'_>;
}
#[derive(Debug,DbInsertable)]
struct Dog {
    hobby: String,
    age: i32,
    name: String,

}
/// 忽略未使用的字段提示
#[allow(dead_code)]
#[derive(FromRow, Debug, DbInsertable, Default)]
pub struct TaskEntity {
    id: u32,
    err_msg: String,
    hash: String, // taskId
    total: u32,
    finished: u32,
    state: u32,
    file_name: String,
    media_play_list: String,
    key: Box<[u8]>,
    iv: Box<[u8]>,
    key_num: u32,

    headers: String,
    url_prefix: Option<String>,
    save_path: String,
    temp_path: String,
    no_combine: bool,
}
impl From<&M3u8Entity> for TaskEntity {
    fn from(entity: &M3u8Entity) -> Self {
        let mut def = TaskEntity::default();
        def.hash = entity.temp_path.clone();
        def.total = entity.clip_num() as u32;
        def.state = Signal::Normal as u32;
        def.file_name = entity.save_path.clone();
        def.media_play_list = {
            let mut s = Vec::new();
            entity.media_play_list.write_to(&mut s).unwrap();
            String::from_utf8_lossy(&s).to_string()
        };
        def.key = entity.key.into();
        def.iv = entity.iv.into();
        def.key_num = entity.key_num as u32;

        def.headers = encode_headers(&entity.headers);
        def.url_prefix = entity.url_prefix.clone();
        def.save_path = entity.save_path.clone();
        def.temp_path = entity.temp_path.clone();
        def.no_combine = entity.no_combine;

        def
    }
}
impl Into<TaskState> for TaskEntity {
    fn into(self) -> TaskState {
        let mut en: M3u8Entity = M3u8Entity::default();
        
        // en.total = row.try_get::<usize,_>(3)?;
        en.content = self.media_play_list;
        // en.key.copy_from_slice( row.try_get::<&[u8],_>("key")?);
        en.key.copy_from_slice(&self.key);
        en.iv.copy_from_slice(&self.iv);
        en.key_num = self.key_num as usize;

        en.headers = decode_headers(self.headers);
        en.url_prefix = self.url_prefix;
        en.save_path = self.save_path;
        en.temp_path = self.temp_path;
        en.no_combine = self.no_combine;

        let mut task = TaskState::from(&en);
        task.set_ext(
            self.finished as usize,
            // self.state,
            (self.state as usize).into(),
            &self.err_msg,
        );
        
        task
    }
}

pub fn get_conn() -> &'static Pool<Sqlite>{
    &POOL
}
pub async fn init_table(db: &Pool<Sqlite>) -> Result<(), sqlx::Error>{
    // 执行 SQL 语句来创建表
    sqlx::query(CREATE_TABLE_SQL)
       .execute(db)
       .await?;
    Ok(())
}

#[tokio::test] // Requires the `attributes` feature of `async-std`
async fn test_create_table() -> Result<(), sqlx::Error> {
    let db = get_conn();

    // 执行 SQL 语句来创建表
    let res = sqlx::query(CREATE_TABLE_SQL)
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
#[test]
fn test_macro() {
    let dog = Dog{name: "Dog".to_string(), age: 1, hobby: "hobby".to_string()};

    let ins = Dog::insert_statement();
    println!("ins: {}", ins);

    let st = dog.as_arguments();
    println!("st: {:?}", st);

}
