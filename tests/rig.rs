use rigctld::{Daemon, Mode, Rig};
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

macro_rules! tokio {
    ($e:expr) => {
        Runtime::new().unwrap().block_on(async { $e })
    };
}

#[test]
fn lifecycle() {
    tokio!({
        let daemon = Daemon::default();
        let mut rigctld = daemon.spawn().await.unwrap();

        sleep(Duration::from_millis(250)).await;

        assert!(rigctld.is_running().unwrap());

        let mut rig = Rig::new(daemon.get_host(), daemon.get_port());
        rig.connect().await.unwrap();
        assert_eq!(rig.disconnect(), true);

        rigctld.kill().await.unwrap();
    })
}

#[test]
fn deamon_not_running() {
    tokio!({
        let mut rig = Rig::new("127.0.0.1", 4532);
        assert_eq!(rig.connect().await.is_err(), true);
    })
}

#[test]
fn rig_frequency() {
    tokio!({
        let daemon = Daemon::default();
        let mut rigctld = daemon.spawn().await.unwrap();

        sleep(Duration::from_millis(250)).await;

        let mut rig = Rig::new(daemon.get_host(), daemon.get_port());
        rig.connect().await.unwrap();

        let freq_before = rig.get_frequency().await.unwrap();
        rig.set_frequency(7123000).await.unwrap();
        let freq_after = rig.get_frequency().await.unwrap();

        assert_ne!(freq_before, 7123000);
        assert_eq!(freq_after, 7123000);

        rigctld.kill().await.unwrap();
    })
}

#[test]
fn rig_mode() {
    tokio!({
        let daemon = Daemon::default();
        let mut rigctld = daemon.spawn().await.unwrap();

        sleep(Duration::from_millis(250)).await;

        let mut rig = Rig::new(daemon.get_host(), daemon.get_port());
        rig.connect().await.unwrap();

        let (mode_before, pb_before) = rig.get_mode().await.unwrap();
        rig.set_mode(Mode::LSB, 1234).await.unwrap();
        let (mode_after, pb_after) = rig.get_mode().await.unwrap();

        assert_ne!(mode_before, Mode::LSB);
        assert_ne!(pb_before, 1234);
        assert_eq!(mode_after, Mode::LSB);
        assert_eq!(pb_after, 1234);

        rigctld.kill().await.unwrap();
    })
}

#[test]
#[ignore]
fn device_icom_ic7200() {
    tokio!({
        let daemon = Daemon::default()
            .set_model(3061)
            .set_serial_speed(19200)
            .set_civ_address(0x76)
            .set_rig_file("/dev/ttyUSB0".into());
        let mut rigctld = daemon.spawn().await.unwrap();

        sleep(Duration::from_millis(1000)).await;

        let mut rig = Rig::new(daemon.get_host(), daemon.get_port());
        rig.connect().await.unwrap();

        rig.get_frequency().await.unwrap();
        rig.get_mode().await.unwrap();

        rigctld.kill().await.unwrap();
    })
}
