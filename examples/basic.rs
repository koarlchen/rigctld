use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let mut rig = rigctld::Rigctld::new("127.0.0.1", 8001);
    rig.connect().await.unwrap();

    let mut counter = 7100000;
    loop {
        rig.set_frequency(counter).await.unwrap();
        rig.set_mode(rigctld::RigctldMode::LSB, 0).await.unwrap();
        let freq = rig.get_frequency().await.unwrap();
        let (mode, pb) = rig.get_mode().await.unwrap();
        println!("{} Hz", freq);
        println!("Mode: {}, Passband: {}", mode, pb);
        sleep(Duration::from_millis(500)).await;
        counter += 1;
    }
}
