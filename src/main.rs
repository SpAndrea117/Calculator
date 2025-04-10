use internal::estimate_expression;
use log::{LevelFilter, error, info};
use signal_hook::{consts::SIGINT, iterator::Signals};
use simple_logger::SimpleLogger;
use std::{io, str::FromStr, sync::mpsc, thread};

mod internal;

fn main() -> io::Result<()> {
    let log_level = LevelFilter::from_str(std::env::var("RUST_LOG").unwrap_or_default().as_str())
        .unwrap_or(LevelFilter::Off);
    SimpleLogger::new()
        .with_level(log_level)
        .with_colors(true)
        .init()
        .unwrap_or_default();

    let mut signals = Signals::new([SIGINT])?;
    let (termination_tx, termination_rx) = mpsc::channel::<()>();

    // Thread for handling termination signal
    thread::spawn(move || {
        for sig in signals.forever() {
            info!("Received signal {:?}", sig);
            let _ = termination_tx.send(());
            return;
        }
    });

    // Thread for handling business logic
    thread::spawn(move || {
        loop {
            let mut buf = String::new();
            println!("Waiting for user input:");
            match io::stdin().read_line(&mut buf) {
                Ok(_) => {
                    info!("Input data -> {}", buf.trim());
                    match estimate_expression(&buf.trim()) {
                        Ok(res) => println!("Result of expression {} is {res}", buf.trim()),
                        Err(e) => println!("Cannot estimate expression due to error {e}"),
                    }
                }
                Err(e) => error!("Error reading input data {e}"),
            };
        }
    });

    match termination_rx.recv() {
        Ok(_) => return Ok(()),
        Err(e) => {
            info!("Error receiving termination signal {e}. Killing process...");
            return Ok(());
        }
    }
}
