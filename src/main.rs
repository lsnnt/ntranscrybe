use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::{FftFixedIn, Resampler};
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
        let mut resampler =
            FftFixedIn::<f32>::new(48000, 16000, 1024, 1, 1).expect("failed to create resampler");

        let mut audio_buffer: Vec<f32> = Vec::new();
        let mut resample_buffer: Vec<f32> = Vec::new();
        let sample_rate = 16000;
        let process_interval = sample_rate * 3; // run every 3 seconds
        let overlap = sample_rate * 2; // keep last 2 seconds
        let window_size = sample_rate * 10; // max 10 second context

        let mut new_samples_counter = 0;
        let mut previous_text = String::new();
        let mut state = ctx.create_state().expect("failed to create state");
        while let Ok(raw_audio_data) = rx.recv() {
            // stereo -> mono
            let mono_48khz: Vec<f32> = raw_audio_data
                .chunks_exact(2)
                .map(|frame| (frame[0] + frame[1]) * 0.5)
                .collect();

            // accumulate until we have enough samples for rubato
            resample_buffer.extend(mono_48khz);

            while resample_buffer.len() >= 1024 {
                let chunk: Vec<f32> = resample_buffer.drain(..1024).collect();

                let output = match resampler.process(&[chunk], None) {
                    Ok(output) => output,
                    Err(err) => {
                        eprintln!("Resampler error: {}", err);
                        break;
                    }
                };

                let mono_16khz = &output[0];
                new_samples_counter += mono_16khz.len();
                audio_buffer.extend_from_slice(mono_16khz);
            }

            if new_samples_counter >= process_interval {
                if audio_buffer.len() > window_size {
                    let excess = audio_buffer.len() - window_size;
                    audio_buffer.drain(..excess);
                }

                if state.full(params.clone(), &audio_buffer).is_ok() {
                    let mut current_text = String::new();

                    for segment in state.as_iter() {
                        current_text.push_str(&format!("{} ", segment));
                    }

                    let current_text = current_text.trim().to_string();

                    // print only new text
                    if current_text.starts_with(&previous_text) {
                        let new_part = current_text[previous_text.len()..].trim();

                        if !new_part.is_empty() {
                            print!("{} ", new_part);
                            std::io::stdout().flush().unwrap();
                        }
                    } else {
                        // whisper revised previous tokens
                        println!("\n{}", current_text);
                    }

                    previous_text = current_text;

                    // keep only overlap audio
                    if audio_buffer.len() > overlap {
                        audio_buffer.drain(..audio_buffer.len() - overlap);
                    }
                }

                new_samples_counter = 0;
            };
        }
    });

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
