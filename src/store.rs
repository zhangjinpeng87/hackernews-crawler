use mysql::prelude::Queryable;
use mysql::*;

use serde::{Deserialize, Serialize};

pub struct Store {
    pool: Pool,
}

impl Store {
    pub fn new(host: &str, db: &str, port: u32) -> Self {
        let url = format!("mysql://newscrawler:newscrawler@{}:{}/{}", host, port, db);
        let pool = Pool::new(Opts::from_url(&url).unwrap()).expect("connect to db failed");

        Self { pool }
    }

    pub fn current_maxitem(&mut self) -> Result<u32> {
        let mut conn = self.pool.get_conn().unwrap();
        conn.query_first("select maxid from maxitem where id = 1")?
            .map_or(Ok(0), |v| match v {
                Value::Int(item_id) => Ok(item_id as u32),
                Value::Bytes(id) => Ok(std::str::from_utf8(&id).unwrap().parse::<u32>().unwrap()),
                _ => Ok(0),
            })
    }

    pub fn update_maxitem(&mut self, maxitem_id: u32) -> Result<()> {
        let mut conn = self.pool.get_conn().unwrap();
        conn.exec_drop("update maxitem set maxid = ? where id = 1", (maxitem_id,))
    }

    // pub fn update_topstories(&mut self, items_id: Vec<u32>) -> Result<()> {
    //     // let mut conn = self.pool.get_conn().unwrap();
    //     // conn.exec_batch(stmt: S, params: I)
    //     Ok(())
    // }

    pub fn insert_new_items(&mut self, items: Vec<Item>) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        let mut conn = self.pool.get_conn().unwrap();
        conn.exec_batch(
            r"INSERT IGNORE INTO items (id, deleted, type, who, time, dead, kids, title, content, score, url, parent)
            VALUES (:id, :deleted, :type, :who, :time, :dead, :kids, :title, :content, :score, :url, :parent)", 
            items.iter().map(|item| params!{
                "id" => item.id,
                "deleted" => item.deleted,
                "type" => item.tp.clone(),
                "who" => item.who.clone(),
                "time" => item.time,
                "dead" => item.dead,
                "kids" => format!("{:?}", item.kids),
                "title" => item.title.clone(),
                "content" => item.text.clone(),
                "score" => item.score,
                "url" => item.url.clone(),
                "parent" => item.parent,
            })
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    #[serde(default)]
    pub deleted: bool,
    #[serde(rename = "type")]
    pub tp: String,
    #[serde(rename = "by")]
    #[serde(default)]
    pub who: String,
    pub time: u32,
    #[serde(default)]
    pub dead: bool,
    #[serde(default)]
    pub kids: Vec<u32>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub score: u32,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub parent: u32,
}

impl From<String> for Item {
    fn from(s: String) -> Self {
        let p: Item = serde_json::from_str(&s).unwrap();
        p
    }
}
