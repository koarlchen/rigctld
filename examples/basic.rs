use rigctld::{Daemon, Rig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Start `rigctld`
    let mut d = Daemon::default();
    println!("rigctld version: {}", d.get_version().await.unwrap());
    d.spawn().await.unwrap();

    // Wait a few milliseconds until `rigctld` is ready
    sleep(Duration::from_millis(250)).await;

    // Connect to `rigctld`
    let mut rig = Rig::new(d.get_host(), d.get_port());
    rig.connect().await.unwrap();

    // Set mode
    let (mode, _) = rig.get_mode().await.unwrap();
    println!("Rig started in mode {}", mode);
    rig.set_mode(rigctld::Mode::LSB, 0).await.unwrap();
    let (mode, _) = rig.get_mode().await.unwrap();
    println!("Set rig to mode {}", mode);

    let mut counter = 7000000;
    while counter < 7200000 {
        // Set frequency
        rig.set_frequency(counter).await.unwrap();

        // Get frequency
        let freq = rig.get_frequency().await.unwrap();
        println!("Current frequency {} Hz", freq);

        counter += 10000;

        sleep(Duration::from_millis(500)).await;
    }

    println!("Done.");
}
