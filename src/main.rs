use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::io::Write;
use std::sync::mpsc::channel;
use std::thread;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    // model loading
    let path = std::env::args().nth(1).expect("No path given");
    // turn off verbose logging
    whisper_rs::install_logging_hooks();
    let ctx = WhisperContext::new_with_params(path, WhisperContextParameters::default())
        .expect("failed to load model");
    let mut params = FullParams::new(SamplingStrategy::BeamSearch {
        beam_size: 5,
        patience: -1.0,
    });


    // we also explicitly disable anything that prints to stdout
    // despite all of this you will still get things printing to stdout,
    // be prepared to deal with it
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_translate(true);

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
    let (tx, rx) = channel();

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
        let mut audio_buffer: Vec<f32> = Vec::new();
        let sample_rate = 16000;
        let process_interval = sample_rate * 3; // Process every 3 seconds of audio
        let max_window_size = sample_rate * 30; // Keep max 30 seconds of context
        let mut state = ctx.create_state().expect("failed to create state");
        let mut new_samples_counter = 0;

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
                // println!("Mono 16khz: {:?}", mono_16khz);
            }
            new_samples_counter += mono_16khz.len();
            audio_buffer.extend(mono_16khz);

            // Draining addition chunks after 30s

            if new_samples_counter >= process_interval {
                if audio_buffer.len() > max_window_size {
                    let drain_amount = audio_buffer.len() - max_window_size;
                    audio_buffer.drain(0..drain_amount);
                }
                if let Ok(_) = state.full(params.clone(), &audio_buffer[..]) {
                    // Use a carriage return `\r` to cleanly refresh the line with updated translations
                    for segment in state.as_iter() {
                        println!(
                            "[{} - {}]: {}",
                            // note start and end timestamps are in centiseconds
                            // (10s of milliseconds)
                            segment.start_timestamp(),
                            segment.end_timestamp(),
                            // the Display impl for WhisperSegment will replace invalid UTF-8 with the Unicode replacement character
                            segment
                        );
                    }

                    std::io::stdout().flush().unwrap();
                }

                new_samples_counter = 0;
            };
        }
    });

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
