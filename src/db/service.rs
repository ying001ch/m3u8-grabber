use std::{ops::Deref, sync::LazyLock, thread};

use sqlx::{prelude::Type, ColumnIndex, Decode, FromRow, Pool, Row, Sqlite};

use crate::{async_runtime, config::{Signal, TaskState}, db::{get_conn, util::encodeHeaders}, M3u8Item::M3u8Entity};

use super::util::decodeHeaders;


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
        .bind(0) //finished
        .bind(Signal::Normal as u32) // state
        .bind(entity.save_path.as_str()) 
        .bind({
            let mut s = Vec::new();
            entity.media_play_list.write_to(&mut s).unwrap();
            String::from_utf8_lossy(&s).to_string()
        }) // playList
        .bind(&entity.key[..]) // key
        .bind(&entity.iv[..]) // iv
        .bind(entity.key_num as u32) // key_num
        .bind(encodeHeaders(&entity.headers)) // headers
        .bind(entity.url_prefix.as_ref()) // url_prefix
        .bind(entity.save_path.as_str()) // save_path
        .bind(entity.temp_path.as_str()) // temp_path
        .bind(entity.no_combine) // no_combine
        .execute(get_conn())
        .await?;
    println!("last_insert_rowid: {:?}", res.last_insert_rowid());

    Ok(())
}
pub async fn update_state(hash: &str, sign: Signal, finished: u32, err_msg: Option<String>) -> anyhow::Result<u64> {
    let res = sqlx::query("update TaskEntity 
            set state = ?, err_msg=? , finished =?
             where hash = ?")
       .bind(sign as i32)
       .bind(err_msg.unwrap_or_default())
       .bind(finished)
       .bind(hash)
       .execute(get_conn())
       .await?;
    println!(" res rows_affected : {:?}", res.rows_affected());

    Ok(res.rows_affected()) 
}
pub async fn del_task(hash: &str) -> anyhow::Result<u64> {
    //TODO 
    let res = sqlx::query("delete from TaskEntity where temp_path =?")
      .bind(hash)
      .execute(get_conn())
      .await?;
    println!(" res rows_affected : {:?}", res.rows_affected());

    Ok(res.rows_affected()) 
}
pub async fn list_all() -> anyhow::Result<Vec<TaskState>> {
    let res = sqlx::query_as::<_,TaskState>("select * from TaskEntity
            order by id desc")
         .fetch_all(get_conn())
        .await?;

    Ok(res)
}
pub async fn list_task(state: Signal) -> anyhow::Result<Vec<TaskState>> {
    let res = sqlx::query_as::<_,TaskState>("select * from TaskEntity
            where state = ?
            order by id desc")
        .bind(state as u32)
         .fetch_all(get_conn())
        .await?;

    Ok(res)
}
#[tokio::test]
pub async fn tests_add() -> anyhow::Result<()> {
    let mut en = M3u8Entity::default();
    en.temp_path = "456".to_string();
    en.save_path = "798".to_string();
    en.headers = vec![("agent".to_string(), "postman".to_string())];
    en.url_prefix = Some("https://baidu.com".to_string());
    en.key = [1; 16];
    en.iv = [2; 16];
    en.key_num = 7;
    en.no_combine = true;

    add_task(&en).await?;

    let list = list_task(Signal::Normal).await?;
    list.iter().for_each(|f|{
        println!("row : {:?}", f);
    });

    Ok(())
}
#[tokio::test]
pub async fn tests_update() -> anyhow::Result<()> {
    update_state("456", Signal::End, 10, None).await?;
    
    let list = list_task(Signal::End).await?;
    list.iter().for_each(|f|{
        println!("row : {:?}", f);
    });

    Ok(())
}
#[tokio::test]
pub async fn tests_del() -> anyhow::Result<()> {
    del_task("").await?;
    
    let list = list_task(Signal::Pause).await?;
    println!("list : {:?}", list);

    Ok(())
}