use reqwest;

use std::sync::mpsc::Receiver;

use std::time::Duration;

use crate::store::{Item, Store};

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

    fn fetch_res_by_uri(&self, query: &str) -> Result<String, reqwest::Error> {
        reqwest::blocking::get(&(self.base_uri.clone() + query))?.text()
    }

    // Fetch items between (start, end]
    pub fn fetch_items_between(&self, start: u32, end: u32) -> Vec<Item> {
        let mut res = vec![];
        for i in start..end + 1 {
            match self.fetch_item(i) {
                Ok(response) => {
                    println!("resp: {}", response);
                    if response.len() < 10 {
                        println!("invalid response");
                    } else {
                        let item = Item::from(response);
                        res.push(item);
                    }
                }
                Err(e) => {}
            }
        }
        res
    }
}

pub struct Crawler {
    hub: NewsHub,
    closer: Receiver<u32>,
    store: Store,
}

impl Crawler {
    pub fn new(base_uri: &str, host: &str, db: &str, port: u32, closer: Receiver<u32>) -> Self {
        let hub = NewsHub::new(base_uri);
        let store = Store::new(host, db, port);
        Self { hub, closer, store }
    }

    pub fn grab_recent_events(&mut self) {
        let new_max_id = match self.hub.fetch_maxitem() {
            Ok(resp) => resp.trim().parse::<u32>().unwrap(),
            Err(_) => {
                return;
            }
        };

        let old_max_id = match self.store.current_maxitem() {
            Ok(id) => id,
            Err(e) => {
                return;
            }
        };

        if old_max_id >= new_max_id {
            return;
        }

        let items = self.hub.fetch_items_between(old_max_id + 1, new_max_id);

        match self.store.insert_new_items(items) {
            Ok(()) => {
                let _ = self.store.update_maxitem(new_max_id);
            }
            Err(e) => {
                println!("insert new items error: {:?}", e);
            }
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

            self.grab_recent_events();
        }
    }
}
