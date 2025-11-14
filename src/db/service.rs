
use crate::{config::{Signal, TaskState}, db::{get_conn, DbInsertable, TaskEntity}, M3u8Item::M3u8Entity};

/// Add a new task to the database
pub async fn add_task(entity: &M3u8Entity) -> anyhow::Result<()> {
    let task_entity = TaskEntity::from(entity);
    let argus = task_entity.as_arguments();
    // let mut qeryas = sqlx::query(TaskEntity::insert_statement().as_str());
    // for a in argus.{
    //     qeryas = qeryas.bind(a);
    // }
    let res = sqlx::query_with(TaskEntity::insert_statement().as_str(), argus)
        .execute(get_conn().await)
        .await?;
    println!("last_insert_rowid: {:?}", res.last_insert_rowid());

    Ok(())
}
/// update task state
pub async fn update_state(hash: &str, sign: Signal, finished: u32, err_msg: Option<String>) -> anyhow::Result<u64> {
    let res = sqlx::query("update TaskEntity 
            set state = ?, err_msg=? , finished =?
             where hash = ?")
       .bind(sign as i32)
       .bind(err_msg.unwrap_or_default())
       .bind(finished)
       .bind(hash)
       .execute(get_conn().await)
       .await?;
    println!(" res rows_affected : {:?}", res.rows_affected());

    Ok(res.rows_affected()) 
}
/// 删除任务
pub async fn del_task(hash: &str) -> anyhow::Result<u64> {
    //TODO 
    let res = sqlx::query("delete from TaskEntity where temp_path =?")
      .bind(hash)
      .execute(get_conn().await)
      .await?;
    println!(" res rows_affected : {:?}", res.rows_affected());

    Ok(res.rows_affected()) 
}
/// 列出所有任务
pub async fn list_all() -> anyhow::Result<Vec<TaskState>> {
    let res = sqlx::query_as::<_,TaskEntity>("select * from TaskEntity
            order by id desc")
         .fetch_all(get_conn().await)
        .await?
        .into_iter()
        .map(|entity| entity.into())
        .collect();

    Ok(res)
}
#[allow(dead_code)]
pub async fn list_task(state: Signal) -> anyhow::Result<Vec<TaskEntity>> {
    let res = sqlx::query_as::<_,TaskEntity>("select * from TaskEntity
            where state = ?
            order by id desc")
        .bind(state as u32)
         .fetch_all(get_conn().await)
        .await?;

    Ok(res)
}
#[cfg(test)]
mod test {
    use crate::{M3u8Item::M3u8Entity, config::Signal, db::service::{add_task, del_task, list_task, update_state}};

    #[tokio::test]
    pub async fn tests_add() -> anyhow::Result<()> {
        let mut en = M3u8Entity::default();
        en.temp_path = "--Asd123+++".to_string();
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
        update_state("--Asd123+++", Signal::End, 10, None).await?;
        
        let list = list_task(Signal::End).await?;
        list.iter().for_each(|f|{
            println!("row : {:?}", f);
        });

        Ok(())
    }
    #[tokio::test]
    pub async fn tests_del() -> anyhow::Result<()> {
        del_task("--Asd123+++").await?;
        
        let list = list_task(Signal::Pause).await?;
        println!("list : {:?}", list);

        Ok(())
    }
    #[tokio::test]
    pub async fn tests_list() -> anyhow::Result<()> {
        let list = list_task(Signal::End).await?;
        // let list = list_all().await?;
        list.iter().for_each(|f|{
            println!("row : {:?}", f);
        });

        Ok(())
    }
}