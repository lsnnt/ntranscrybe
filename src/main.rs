use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::channel;
use std::thread;

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
        move |data: &[f32], _| {
            let _ = tx.send(data.to_vec());
        },
        err_fn,
        None,
    )?;
    stream.play()?;
    thread::spawn(move || {
        // let mut audio_buffer: Vec<f32> = Vec::new();

        while let Ok(raw_audio_data) = rx.recv() {
        //     Downsampling the data (usually in 48Khz needed 16Khz)
        //     48000 - > 16000 Hz
            let mut mono_16khz = Vec::with_capacity(raw_audio_data.len() / 6);
            for chunk in raw_audio_data.chunks_exact(6) {
                // Downmix the first stereo frame of this group to mono
                let mono1 = (chunk[0] + chunk[1]) / 2.0;
                let mono2 = (chunk[2] + chunk[3]) / 2.0;
                let mono3 = (chunk[4] + chunk[5]) / 2.0;

                // Average the 3 mono samples to get 1 downsampled sample
                let final_sample = (mono1 + mono2 + mono3) / 3.0;


                mono_16khz.push(final_sample); // Decimates by 3 automatically by skipping the other 2 frames
                println!("Mono 16khz: {:?}", mono_16khz);
            }


        }
    });

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
