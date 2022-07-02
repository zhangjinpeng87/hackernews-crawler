use crate::store::{Item, Store};
use reqwest;
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
        }
    }

    // Fetch items between [start, end] concurrently
    fn fetch_items_between(&self, start: u32, end: u32) -> Vec<Item> {
        let (s, r) = mpsc::channel();
        for i in start..end + 1 {
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
                    Err(_) => { let _ = sender.send(None).unwrap(); }
                }
            });
        }

        let mut res = vec![];
        for _ in start..end + 1 {
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

            let items = self.fetch_items_between(old_max_id + 1, next_id);
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
        }
    }
}
