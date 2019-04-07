extern crate chrono;
extern crate flexi_logger;
extern crate log;
extern crate warp;

use chrono::{DateTime, Local};
use log::Record;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use warp::{http::Response, Filter};

type DB = Arc<Mutex<State>>;

static SAVE_FILE: &str = "clicks.txt";

struct State {
    clicks: u64,
    last_saved: DateTime<Local>,
}

impl State {
    // TODO: the path should not be hardcoded
    /// Load the clicks from the file `clicks.txt`
    fn from_file() -> State {
        // Open the file or create if not exist
        let mut file = match File::open(SAVE_FILE) {
            Err(_) => {
                // The file does not exist so create one
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
        State {
            clicks: counter,
            last_saved: Local::now(),
        }
    }
    /// Save the clicks to the file `clicks.txt`
    fn to_file(&mut self) -> std::io::Result<()> {
        // Check if 10s have passed since last save
        if Local::now()
            .signed_duration_since(self.last_saved)
            .num_seconds()
            > 10
        {
            self.last_saved = Local::now();
            let mut file = File::create(SAVE_FILE)?;
            file.write_all(&self.clicks.to_string().into_bytes())?;
        }

        Ok(())
    }
}

fn main() {
    // Create a in memeory database
    let db = Arc::new(Mutex::new(State::from_file()));
    let db = warp::any().map(move || db.clone());

    // Initialise the logger
    flexi_logger::Logger::with_str("all")
        .format(custom_format)
        .start()
        .unwrap();

    // GET / => serve the static folder
    let index = warp::get2().and(warp::any().and(warp::fs::dir("static")));

    // GET /click => get the git counter and increment by one
    let click = warp::get2().and(warp::path("click").and(db.clone().map(get_clicks_and_increment)));
    let routes = click.or(index);
    let routes = routes.with(warp::log("all"));

    // Start the server
    let addr = "0.0.0.0:8000";
    println!("Server started at: {}", addr);
    let mut addr_iter = addr.to_socket_addrs().unwrap();
    warp::serve(routes).run(addr_iter.next().unwrap());
}

/// Return a response with the current click. Also increment the counter by one.
fn get_clicks_and_increment(db: DB) -> impl warp::Reply {
    let mut state = db.lock().unwrap();
    state.clicks += 1;
    state.to_file().unwrap(); // TODO check for error, dont panic
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
