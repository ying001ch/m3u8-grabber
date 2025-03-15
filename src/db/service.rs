use std::{ops::Deref, sync::LazyLock, thread};

use sqlx::{prelude::Type, ColumnIndex, Decode, FromRow, Pool, Row, Sqlite};

use crate::{async_runtime, config::Signal, db::util::encodeHeaders, M3u8Item::M3u8Entity};

use super::util::decodeHeaders;


static POOL: LazyLock<Pool<Sqlite>> = LazyLock::new(||{
    thread::spawn(||{
        async_runtime::block_on(super::get_conn()).unwrap()
    }).join().unwrap()
});
const INSERT_TASK: &str = r#"
INSERT INTO TaskEntity (
    err_msg, hash, total, finished, state, 
    file_name, media_play_list, key, iv, key_num, 
    headers, url_prefix, save_path, temp_path, no_combine
) VALUES (
    ?,?,?,?,?,
    ?,?,?,?,?,
    ?,?,?,?,?
);
"#;
pub async fn add_task(entity: &M3u8Entity) -> anyhow::Result<()> {
    let res = sqlx::query(INSERT_TASK)
        .bind("")
        .bind(entity.temp_path.as_str())
        .bind(entity.clip_num() as u32)
        .bind(0)
        .bind(0) // state
        .bind(entity.save_path.as_str()) 
        .bind(entity.content.as_str()) // playList
        .bind(&entity.key[..]) // key
        .bind(&entity.iv[..]) // iv
        .bind(entity.key_num as u32) // key_num
        .bind(encodeHeaders(&entity.headers)) // headers
        .bind(entity.url_prefix.as_ref()) // url_prefix
        .bind(entity.save_path.as_str()) // save_path
        .bind(entity.temp_path.as_str()) // temp_path
        .bind(entity.no_combine) // no_combine
        .execute(POOL.deref())
        .await?;
    println!("last_insert_rowid: {:?}", res.last_insert_rowid());

    Ok(())
}
pub async fn update_state(hash: &str, sign: Signal) -> anyhow::Result<()> {
    //TODO 

    Ok(()) 
}
pub async fn list_task(state: i32) -> anyhow::Result<Vec<M3u8Entity>> {
    let res = sqlx::query_as::<_,M3u8Entity>("select * from TaskEntity
            where state = ?
            order by id desc")
        .bind(state)
         .fetch_all(POOL.deref())
        .await?;
    

    Ok(res)
}
#[tokio::test]
pub async fn tests_add() -> anyhow::Result<()> {
    let en = M3u8Entity::default();
    add_task(&en).await?;

    let list = list_task(0).await?;
    println!("list : {:?}", list);

    Ok(())
}