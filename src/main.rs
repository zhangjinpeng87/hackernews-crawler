pub mod crawler;
pub mod store;
use ctrlc;
use std::sync::mpsc;

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
    // #[default = "https://hacker-news.firebaseio.com/v0"]
    hackernews_addr: String,
}

fn main() {
    let (closer, rcv) = mpsc::channel();

    let args = Args::parse();

    let mut crawler = crawler::Crawler::new(
        &args.hackernews_addr,
        &args.dbhost,
        "hackernews",
        args.dbport,
        rcv,
    );

    let handle = std::thread::spawn(move || {
        crawler.run();
    });

    ctrlc::set_handler(move || {
        closer.send(0).unwrap();
    })
    .expect("Error setting Ctrl-C handler");
    handle.join().expect("wait crawler failed");
}
