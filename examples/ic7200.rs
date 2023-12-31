use rigctld::{Daemon, Rig};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let daemon = Daemon::default()
        .set_model(3061)
        .set_serial_speed(19200)
        .set_civ_address(0x76)
        .set_rig_file("/dev/ttyUSB0".into());
    let _rigctld = daemon.spawn().await.unwrap();

    sleep(Duration::from_millis(1000)).await;

    let mut rig = Rig::new(daemon.get_host(), daemon.get_port());
    rig.set_communication_timeout(Duration::from_millis(1000));
    rig.connect().await.unwrap();

    loop {
        rig.get_frequency().await.unwrap();
        rig.get_mode().await.unwrap();
    }
}
