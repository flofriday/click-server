extern crate chrono;
extern crate flexi_logger;
extern crate log;
extern crate warp;

use chrono::Local;
use log::Record;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::timer::Interval;
use warp::{http::Response, Filter, Future, Stream};

type DB = Arc<Mutex<State>>;

static SAVE_FILE: &str = "clicks.txt";

/// Saves the state of the webapp
struct State {
    clicks: u64,
}

impl State {
    /// Load the clicks from the file `clicks.txt`
    fn from_file() -> State {
        // Open the file or create if not exist
        let mut file = match File::open(SAVE_FILE) {
            Err(_) => {
                // The file does not exist so create one
                print!("No save-file found, create new one");
                File::create(SAVE_FILE).expect("Unable to create the file clicks.txt");
                File::open(SAVE_FILE).expect("Cannot open just created file.")
            }
            Ok(file) => file,
        };

        // Read the content of the file
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Unable to read from file clicks.txt");

        // Create the counter
        let counter: u64 = content.parse().unwrap_or(0);
        State { clicks: counter }
    }
    //TODO: this should not be blocking, rewrite with futures
    /// Save the clicks to the file `clicks.txt`
    fn to_file(&mut self) -> std::io::Result<()> {
        let mut file = File::create(SAVE_FILE)?;
        file.write_all(&self.clicks.to_string().into_bytes())
    }
}

fn main() {
    // Create a in memeory database
    let db_raw = Arc::new(Mutex::new(State::from_file()));
    let db_save_task = db_raw.clone();
    let db_web = warp::any().map(move || db_raw.clone());

    // Initialise the logger
    flexi_logger::Logger::with_str("all")
        .format(custom_format)
        .start()
        .unwrap();

    // GET / => serve the static folder
    let index = warp::get2().and(warp::any().and(warp::fs::dir("static")));

    // GET /click => get the git counter and increment by one
    let click =
        warp::get2().and(warp::path("click").and(db_web.clone().map(get_clicks_and_increment)));
    let routes = click.or(index);
    let routes = routes.with(warp::log("all"));

    // Create a background task that saves the clicks every 5s to a file
    let save_task = Interval::new(Instant::now(), Duration::from_secs(5))
        .for_each(move |_| {
            match db_save_task.lock().unwrap().to_file() {
                Err(e) => eprintln!("Unable to save clicks to file: {}", e),
                _ => (),
            }
            Ok(())
        })
        .map_err(|e| panic!("interval errored; err={:?}", e));

    // Configure the server
    let addr = "0.0.0.0:8000";
    println!("Server started at: {}", addr);
    let addr: SocketAddr = addr.parse().unwrap();
    let server = warp::serve(routes).bind(addr);

    // Start the runtime
    tokio::run(future::lazy(|| {
        tokio::spawn(server);
        tokio::spawn(save_task);
        Ok(())
    }));
}

/// Increment the counter by one and return that new value
fn get_clicks_and_increment(db: DB) -> impl warp::Reply {
    let mut state = db.lock().unwrap();
    state.clicks += 1;
    Response::builder().body(state.clicks.to_string())
}

/// Custom format for the logger for release mode (more information)
#[cfg(not(debug_assertions))]
fn custom_format(w: &mut io::Write, record: &Record) -> Result<(), io::Error> {
    write!(
        w,
        "[{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S %:z"),
        &record.args()
    )
}

/// Custom format for the logger for debug mode (less information)
#[cfg(debug_assertions)]
fn custom_format(w: &mut io::Write, record: &Record) -> Result<(), io::Error> {
    let tmp: String = record.args().to_string();
    let tmp: Vec<&str> = tmp.split('"').collect();

    let out = format!("\"{}\" {} {}", tmp[1].trim(), tmp[2].trim(), tmp[6].trim());
    write!(w, "[{}] {}", Local::now().format("%H:%M:%S"), out)
}
