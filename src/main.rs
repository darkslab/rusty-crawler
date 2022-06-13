use std::process;
use tokio::signal;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::task;
use tokio::time::{self, Duration};

mod chore {
    pub fn new() -> Outcome {
        // DARK_NEXT  implement this!

        // let (shutdown_tx, shutdown_rx): (broadcast::Sender<()>, broadcast::Receiver<()>) =
        //     broadcast::channel(1);
        // let (wait_tx, wait_rx): (mpsc::Sender<()>, mpsc::Receiver<()>) = mpsc::channel(1);
        // let (output_tx, output_rx): (mpsc::Sender<String>, mpsc::Receiver<String>) =
        //     mpsc::channel(4096);

        // let ctx: Context = Context {
        //     shutdown_tx: shutdown_tx.clone(),
        //     wait_tx: wait_tx.clone(),
        //     output_tx: output_tx.clone(),
        // };
        // let out: Outcome = Outcome {
        //     shutdown_tx: Some(shutdown_tx),
        //     shutdown_rx: Some(shutdown_rx),
        //     wait_tx: Some(wait_tx),
        //     wait_rx: Some(wait_rx),
        //     output_tx: Some(output_tx),
        //     output_rx: Some(output_rx),
        // };
        // (ctx, out)
    }

    pub struct Outcome {
        // DARK_NEXT  implement this!

        // shutdown_tx: Option<broadcast::Sender<()>>,
        // shutdown_rx: Option<broadcast::Receiver<()>>,
        // wait_tx: Option<mpsc::Sender<()>>,
        // wait_rx: Option<mpsc::Receiver<()>>,
        // output_tx: Option<mpsc::Sender<String>>,
        // output_rx: Option<mpsc::Receiver<String>>,
    }

    impl Outcome {
        pub fn context(&self) -> Context {
            // DARK_NEXT  implement this!
        }

        pub async fn recv(self) -> Vec<String> {
            // DARK_NEXT  implement this!
        }
    }

    pub struct Context {
        // DARK_NEXT  implement this!

        // shutdown_tx: broadcast::Sender<()>,
        // wait_tx: mpsc::Sender<()>,
        // output_tx: mpsc::Sender<String>,
    }

    impl Context {
        pub fn spawn<F>(self, future: F)
        where
            F: Future<Output = String> + Send + 'static,
        {
            // DARK_NEXT  implement this!

            // let mut shutdown_rx = self.shutdown_tx.subscribe();
            // task::spawn(async move {
            //     tokio::select! {
            //         // _ = clone_shutdown_rx.recv() => {}
            //         _ = future => {}
            //     }
            //     // drop(clone_wait_tx);
            // });
        }
    }

    impl Clone for Context {
        pub fn clone(&self) -> Self {
            // DARK_NEXT  implement this!
        }
    }
}

#[tokio::main]
async fn main() {

    // ################################################################

    let mut out: chore::Outcome = chore::new();
    out.context().spawn(async move {
        time::sleep(Duration::from_millis(1000)).await;
        String::from("Task finished!")
    });
    let res: Vec<String> = out.recv().await;

    // ----------------------------------------------------------------

    let mut out: chore::Outcome = chore::new();
    for i in 0..10 {
        out.context().spawn(async {
            time::sleep(Duration::from_millis(1000 * i)).await;
            String::from("Task #{} finished!", i)
        });
    }
    let res: Vec<String> = out.recv().await;

    // ----------------------------------------------------------------

    let mut out: chore::Outcome = chore::new();
    let ctx_prev: chore::Context = out.context();
    let ctx_next: chore::Context = ctx_prev.clone();
    ctx_prev.spawn(async move {
        let ctx_prev: chore::Context = ctx_next.clone();
        ctx_next.spawn(async move {
            let ctx_next: chore::Context = ctx_prev.clone();
            ctx_prev.spawn(async move {
                ctx_next.spawn(async {
                    time::sleep(Duration::from_millis(1000)).await;
                    String::from("Task >>>> finished!")
                });
                time::sleep(Duration::from_millis(1000)).await;
                String::from("Task >>> finished!")
            });
            time::sleep(Duration::from_millis(1000)).await;
            String::from("Task >> finished!")
        });
        time::sleep(Duration::from_millis(1000)).await;
        String::from("Task > finished!")
    });
    let res: Vec<String> = out.recv().await;

    // ################################################################

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
