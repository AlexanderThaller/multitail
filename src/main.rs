use async_std::channel::Sender;
use async_std::fs::File;
use async_std::io;
use async_std::io::prelude::BufReadExt;
use async_std::io::BufReader;
use async_std::path::Path;
use async_std::prelude::*;
use async_std::task;
use std::time::Duration;

#[async_std::main]
async fn main() {
    let (sender, receiver) = async_std::channel::bounded(100);

    { 0..=10000 }
        .map(|i| format!("file_{}.log", i))
        .for_each(|path| {
            async_std::task::spawn(tail(path, sender.clone()));
        });

    let mut stdout = io::stdout();

    loop {
        let line = receiver.recv().await.unwrap();
        stdout.write_all(line.as_bytes()).await.unwrap();
    }
}

async fn tail(path: impl AsRef<Path>, sender: Sender<String>) {
    // Need to wait until file exists
    let file = loop {
        match File::open(path.as_ref()).await {
            Ok(f) => break f,
            Err(_) => {
                task::sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
    };

    let mut reader = BufReader::new(file);
    let mut read_until = 0u64;

    loop {
        let metadata = async_std::fs::metadata(path.as_ref()).await.unwrap();
        let file_len = metadata.len();

        if read_until < file_len {
            let mut buffer = String::new();
            let read_from_buffer = reader.read_line(&mut buffer).await.unwrap();
            read_until += read_from_buffer as u64;
            sender.send(buffer).await.unwrap();
        } else {
            task::sleep(Duration::from_secs(1)).await;
        }
    }
}
