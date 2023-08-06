use rigctld::{Daemon, Rig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Start `rigctld`
    let daemon = Daemon::default();
    println!("rigctld version: {}", daemon.get_version().await.unwrap());
    let mut rigctld = daemon.spawn().await.unwrap();

    // Wait a few milliseconds until `rigctld` is ready.
    // There is currently no way to check wether the daemon is running and already ready to accept incoming connections.
    // Therefore sleep a while and connect to `rigctld`.
    // The actual duration required to sleep depends on how long it takes for the daemon to connect to the actual rig.
    // This may vary between devices. While using the dummy rig a duration of 250 ms seems reasonable.
    sleep(Duration::from_millis(250)).await;

    // Check wether rigctld is running.
    // This checks only if the process is still up and running and not if the daemon is ready to accept incoming connections.
    // `rigctld` may crash after start if e.g. the requested port is already taken by another process.
    // This happens at runtime and thus the process starts flawlessly at first glance.
    if !rigctld.is_running().unwrap() {
        println!("Failed to start rigctld. Another instance already running?");
        return;
    }

    // Establish connection to `rigctld`
    let mut rig = Rig::new(daemon.get_host(), daemon.get_port());
    rig.connect().await.unwrap();

    // Set and get mode
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

    // Disconnect from `rigctld`
    rig.disconnect();

    // Kill the daemon
    // Not really necessary here since the kernel will kill it anyway after this process ends
    rigctld.kill().await.unwrap();

    println!("Done.");
}
