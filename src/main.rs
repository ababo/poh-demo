use poh::{Event, Hash, PohService};
use rand::{thread_rng, Rng};
use std::time::Duration;
use tokio::{
    join, spawn,
    sync::mpsc::{channel, Receiver, Sender},
    time::sleep,
};

pub mod poh;

#[tokio::main]
async fn main() {
    // Just a random state for the demo.
    let state: Hash = [
        0x95, 0x92, 0x42, 0x42, 0xd2, 0x50, 0x0f, 0x66, 0x90, 0x5e, 0x81, 0xa8, 0xe4, 0x9c, 0x25,
        0x3d, 0xde, 0xb9, 0x36, 0x0c, 0xc5, 0x43, 0xd6, 0xc8, 0x39, 0xd2, 0x5f, 0xc0, 0xa2, 0x3f,
        0x0d, 0xae,
    ];

    let mut service = PohService::new(state);

    let (message_sender, message_receiver) = channel(1);
    let (event_sender, event_receiver) = channel(1);

    let producer = spawn(produce_messages(message_sender));
    let consumer = spawn(consume_events(event_receiver));

    service.process(message_receiver, event_sender).await;
    let (producer_result, consumer_result) = join!(producer, consumer);
    producer_result.unwrap();
    consumer_result.unwrap();
}

async fn produce_messages(messages: Sender<Vec<u8>>) {
    for _ in 0..1000 {
        let delay = thread_rng().gen_range(10..=100);
        sleep(Duration::from_millis(delay)).await;

        let length = thread_rng().gen_range(100..=1000);
        let message = (0..length).map(|_| thread_rng().gen::<u8>()).collect();
        messages.send(message).await.unwrap();
    }
}

async fn consume_events(mut events: Receiver<Event>) {
    while let Some(event) = events.recv().await {
        use Event::*;
        match event {
            Hash { .. } => {} // Omit hashes to limit the output.
            Message { state, message } => {
                println!(
                    "{:.7} message {:.32}...",
                    hex::encode(state),
                    hex::encode(message)
                );
            }
            Tick { state, tick } => {
                println!("{:.7} tick {tick}", hex::encode(state));
            }
        }
    }
}
