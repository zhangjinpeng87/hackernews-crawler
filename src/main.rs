pub mod crawler;
pub mod store;
use ctrlc;
use std::sync::mpsc;
use tokio::runtime;

use clap::Parser;
#[derive(Parser, Default, Debug)]
#[clap(
    author = "zhangjinpeng1987",
    version,
    about = "hackernews events crawler"
)]
struct Args {
    dbhost: String,
    dbport: u32,
    db: String,
    usr: String,
    pwd: String,
    // #[default = "https://hacker-news.firebaseio.com/v0"]
    hackernews_addr: String,
}

fn main() {
    let (closer, rcv) = mpsc::channel();
    let args = Args::parse();

    let rt = runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .unwrap();

    let mut crawler = crawler::Crawler::new(
        &args.hackernews_addr,
        &args.dbhost,
        &args.db,
        args.dbport,
        &args.usr,
        &args.pwd,
        rcv,
        rt,
    );

    ctrlc::set_handler(move || {
        closer.send(0).unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    crawler.run();
}
