use std::process;
use tokio::signal;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    let (shutdown_tx, _): (broadcast::Sender<()>, _) = broadcast::channel(1);
    let (wait_tx, mut wait_rx): (mpsc::Sender<()>, mpsc::Receiver<()>) = mpsc::channel(1);

    let (output_tx, mut output_rx): (mpsc::Sender<String>, mpsc::Receiver<String>) =
        mpsc::channel(4096);
    let (final_tx, final_rx): (oneshot::Sender<Vec<String>>, oneshot::Receiver<Vec<String>>) =
        oneshot::channel();

    for i in 0..10 {
        let mut clone_shutdown_rx = shutdown_tx.subscribe();
        let clone_wait_tx = wait_tx.clone();
        let clone_output_tx = output_tx.clone();

        task::spawn(async move {
            tokio::select! {
                _ = clone_shutdown_rx.recv() => {}
                _ = time::sleep(Duration::from_millis(1000 * i)) => {
                    let _ = clone_output_tx.send(format!("Task #{} finished.", i)).await;
                }
            }
            drop(clone_wait_tx);
        });
    }

    drop(output_tx);
    let clone_wait_tx = wait_tx.clone();
    let mut clone_shutdown_rx = shutdown_tx.subscribe();
    task::spawn(async move {
        let mut output: Vec<String> = Vec::new();
        loop {
            tokio::select! {
                _ = clone_shutdown_rx.recv() => {
                    break;
                }
                result = output_rx.recv() => {
                    match result {
                        Some(line) => {
                            output.push(line);
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
        }
        let _ = final_tx.send(output);
        drop(clone_wait_tx);
    });

    drop(wait_tx);
    tokio::select! {
        _ = wait_rx.recv() => {}
        sigint_result = signal::ctrl_c() => {
            match sigint_result {
                Ok(()) => {
                    if let Err(_) = shutdown_tx.send(()) {
                        eprintln!("Failed to broadcast shutdown message because nobody is listenning, most likely there is no point in waiting. Shutting down...");
                        process::exit(1);
                    }
                    println!();
                    println!("Interrupted!");
                    let _ = wait_rx.recv().await;
                }
                Err(err) => {
                    eprintln!("Failed to listen for SIGINT signal: {}. Shutting down...", err);
                    process::exit(1);
                }
            }
        }
    }

    match final_rx.await {
        Ok(output) => {
            for line in output.into_iter() {
                println!("{}", line);
            }
        }
        Err(_) => {
            eprintln!("Failed to receive the final output. Shutting down...");
            process::exit(1);
        }
    }
}
