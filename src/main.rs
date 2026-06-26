use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::channel;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    let device = host
        .input_devices()?
        .find(|device| device.to_string().contains("BlackHole"))
        .ok_or("Blackhole not found")?;
    println!("Using default input device: {}", &device);
    let config = device.default_input_config()?;

    println!("Using default output config: {:?}", &config);
    // Some things to mind for blackhole usually gives this output config so we are building upon it
    // SupportedStreamConfig { channels: 2, sample_rate: 48000, buffer_size: Range { min: 15, max: 4096 }, sample_format: F32 }
    let streamconfig: cpal::StreamConfig = config.into();
    let (tx,rx) = channel();

    let err_fn = |err| {
        eprintln!("Stream error: {}", err);
    };

    let stream = device.build_input_stream(
        streamconfig,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let _ = tx.send(data.to_vec());
        },
        err_fn,
        None,
    )?;
    stream.play()?;
    std::thread::spawn(move || {
        while let Ok(data) = rx.recv() {
            println!("Data received: {:?}", data);
        }
    });

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
