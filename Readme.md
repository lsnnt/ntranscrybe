# ntranscrybe 
Live captions for any language to english for your mac.
## Demo
<img width="934" height="723" alt="image" src="https://github.com/user-attachments/assets/18dde674-c163-4c3d-9ffe-2abf529509bd" />

## Why this?
You have realised this already exists for windows and linux but for mac you get a live captions by `⌥+⌘+F5` which only supports the english live caption for other languages i can find the most closest as Transcribe on Apple store but its not free (both in freedom and as in beer). So i decided to build one.

## Workflow
<img width="1484" height="1256" alt="image" src="https://github.com/user-attachments/assets/230353cd-e36c-47cd-b32d-6c0153fefdcd" />

## Getting started
1) Install the blackhole 2ch using brew
```shell
brew install blackhole-2ch
```
2) Setup the Audio MIDI

Create a multi output device for the Audio MIDI which redirects the audio to blackhole as well as your speaker
<img width="912" height="592" alt="Screenshot 2026-06-29 at 11 11 57" src="https://github.com/user-attachments/assets/c8aa9f0d-e0ba-4fc4-9e3e-99c3fc1c880c" />

3) Download the ggml multilingual model

  Download it from here :- https://huggingface.co/ggerganov/whisper.cpp

  Chose according to your device memory and how much clarity you want in transcription
  Dont chose the model with .en in it it will be english only 

4) Compile the binary from repo and install it 

```shell
git clone https://github.com/lsnnt/ntranscrybe.git
cd ntranscrybe
cargo build --release
cargo install --path .
```
or (one liner)
```shell
cargo install --git https://github.com/lsnnt/ntranscrybe
```

5) Run it
```
ntranscrybe /path/to/downloaded/model.bin
```

## Whats next?
Next i need help from someone with swift macos development who can integrate it and provide an overlay on the applications for live text 

## Special Thanks
To all my dependencies rubato for audio processing cpal for getting audio and whisper-rs for whisper.cpp rust binding without which the idea may not be possible
