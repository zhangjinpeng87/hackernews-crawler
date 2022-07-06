use crate::store::{Item, Store};
use reqwest;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;

const EVENTS_BATCH_SIZE: u32 = 50;

pub struct NewsHub {
    base_uri: String,
}

impl NewsHub {
    pub fn new(base_uri: impl Into<String>) -> Self {
        Self {
            base_uri: base_uri.into(),
        }
    }

    pub fn fetch_maxitem(&self) -> Result<String, reqwest::Error> {
        self.fetch_res_by_uri("/maxitem.json?print=pretty")
    }

    pub fn fetch_topstories(&self) -> Result<String, reqwest::Error> {
        self.fetch_res_by_uri("/topstories.json?print=pretty")
    }

    pub fn fetch_item(&self, item_id: u32) -> Result<String, reqwest::Error> {
        self.fetch_res_by_uri(&("/item/".to_owned() + &item_id.to_string() + ".json?print=pretty"))
    }

    pub async fn fetch_item_async(&self, item_id: u32) -> Result<String, reqwest::Error> {
        self.fetch_res_by_uri_async(
            &("/item/".to_owned() + &item_id.to_string() + ".json?print=pretty"),
        )
        .await
    }

    pub fn fetch_updates(&self) -> Result<String, reqwest::Error> {
        self.fetch_res_by_uri("/updates.json?print=pretty")
    }

    fn fetch_res_by_uri(&self, query: &str) -> Result<String, reqwest::Error> {
        reqwest::blocking::get(&(self.base_uri.clone() + query))?.text()
    }

    async fn fetch_res_by_uri_async(&self, query: &str) -> Result<String, reqwest::Error> {
        reqwest::get(&(self.base_uri.clone() + query))
            .await?
            .text()
            .await
    }
}

pub struct Crawler {
    hub: Arc<NewsHub>,
    closer: Receiver<u32>,
    store: Store,
    rt: tokio::runtime::Runtime,
    last_updates: Vec<u32>,
}

impl Crawler {
    pub fn new(
        base_uri: &str,
        host: &str,
        db: &str,
        port: u32,
        usr: &str,
        pwd: &str,
        closer: Receiver<u32>,
        rt: tokio::runtime::Runtime,
    ) -> Self {
        let hub = Arc::new(NewsHub::new(base_uri));
        let store = Store::new(host, db, port, usr, pwd);
        Self {
            hub,
            closer,
            store,
            rt,
            last_updates: vec![],
        }
    }

    fn fetch_items(&self, ids: Vec<u32>) -> Vec<Item> {
        let (s, r) = mpsc::channel();
        let ids_cnt = ids.len();
        for i in ids {
            let sender = s.clone();
            let hub = self.hub.clone();
            self.rt.spawn(async move {
                match hub.fetch_item_async(i).await {
                    Ok(response) => {
                        if response.len() < 10 {
                            println!("invalid response");
                            let _ = sender.send(None);
                        } else {
                            let item = Item::from(response);
                            let _ = sender.send(Some(item));
                        }
                    }
                    Err(_) => {
                        let _ = sender.send(None).unwrap();
                    }
                }
            });
        }

        let mut res = vec![];
        for _ in 0..ids_cnt {
            match r.recv_timeout(std::time::Duration::from_secs(60)) {
                Ok(item) => {
                    if let Some(item) = item {
                        res.push(item);
                    }
                }
                Err(_) => {
                    println!("Don't wait since we have waited for 60 seconds");
                    break;
                }
            }
        }

        res
    }

    pub fn grab_recent_events(&mut self) -> bool {
        let new_max_id = match self.hub.fetch_maxitem() {
            Ok(resp) => resp.trim().parse::<u32>().unwrap(),
            Err(_) => {
                return false;
            }
        };

        let mut old_max_id = match self.store.current_maxitem() {
            Ok(id) => id,
            Err(_e) => {
                return false;
            }
        };

        if old_max_id >= new_max_id {
            return false;
        }

        while old_max_id < new_max_id {
            let next_id = std::cmp::min(old_max_id + EVENTS_BATCH_SIZE, new_max_id);

            let items = self.fetch_items((old_max_id + 1..next_id + 1).collect());
            match self.store.insert_new_items(items) {
                Ok(()) => {
                    let _ = self.store.update_maxitem(next_id);
                }
                Err(e) => {
                    println!("insert new items error: {:?}", e);
                }
            }
            old_max_id = next_id;

            // recv close msg
            if self.closer.try_recv().is_ok() {
                return true;
            }
        }

        false
    }

    fn grab_rencent_updates(&mut self) {
        use serde_json::Result;
        let mut updated_items = match self.hub.fetch_updates() {
            Ok(resp) => {
                let updates: Result<Updates> = serde_json::from_str(&resp);
                match updates {
                    Ok(updates) => updates.items,
                    Err(_) => {
                        vec![]
                    }
                }
            }
            Err(_) => {
                return;
            }
        };

        updated_items.sort_unstable();
        // If updated_items is not euqal to last_updates, it means the upstream has
        // refreshed the updates.
        if updated_items != self.last_updates {
            let items = self.fetch_items(updated_items.clone());
            match self.store.update_items(items) {
                Ok(()) => {}
                Err(e) => {
                    println!("update items err: {:?}", e);
                }
            }
            self.last_updates = updated_items;
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.closer.recv_timeout(Duration::from_secs(1)) {
                Ok(_) => {
                    return;
                }
                _ => {}
            }

            let close = self.grab_recent_events();
            if close {
                return;
            }
            self.grab_rencent_updates();
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Updates {
    items: Vec<u32>,
    profiles: Vec<String>,
}
