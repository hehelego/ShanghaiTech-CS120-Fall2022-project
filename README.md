# CS120 Computer Network Report

In each project, we will explain our project part by part. Each part includes a brief explanation and the test method. At the end of each project, we will have an acknowledgement. You can build the code documentation by the following command:

```shell
cargo doc --open
```

[TOC]

## Project 1: Acoustic Link

We have mainly four components:

- Block buffer: The queue-like structure for stream data. Both original version and concurrent version are defined in
  [block_buffer.rs](./proj1_acoustic_link/src/block_buffer.rs).
- Sample I/O Stream: The structures operate on sample streams. Sample streams are divided into instream and outstream. Instreams read samples from physics world and store the data in its buffer. Ostreams fetch samples to its buffer and write them to the physics world. The sample streams are in [samples_stream.rs](./proj1_acoustic_link/src/sample_stream.rs).
- Physics packet sender/receiver: Structures send or receive packet. Sender owns an sample instream and receiver owns an sample outstream. They are responsible for packet detection, modulation and demodulation. The sender and receiver are in [phy_packet.rs](./proj1_acoustic_link/src/phy_packet.rs)
- Physics layer: The assembled physics layer. Provide basic data transmission over acoustic link. There three types of links: PSK, OFDM and PSK with error correction. The physics layers are in [phy_layer.rs](./proj1_acoustic_link/src/phy_layer.rs).

The whole project structure lies below:
![Project structure](./img/structure.png)

### Part 1 Understand your tools

In this part, we implement

- `buffer` and `concurrent_buffer`: Basic block data structure. ([buffer.rs](./proj1_acoustic_link/src/block_buffer/buffer.rs), [concurrent_buffer.rs](./proj1_acoustic_link/src/block_buffer/concurrent_buffer.rs))
- `cpal_stream`: Audio I/O stream. ([cpal_stream.rs](./proj1_acoustic_link/src/sample_stream/cpal_stream.rs))
- `hound_stream`: wav file I/O stream. ([hound_stream.rs](./proj1_acoustic_link/src/sample_stream/hound_stream.rs))

We use `cpal_stream` to realize audio recording and playing. We use `hound_stream` to read audio files.

#### Tests

You can test part1 ck1 by the following command.

```bash
cargo test part1_ck1 --release -- --ignored
```

The program will start recording immediately for 10s.

---

You can test part1 ck2 by

```bash
cargo test part1_ck2 --release -- --ignored
```

The program will play a 10-second music clip while recording at the same time. After 10 seconds, it will replay the recorded sound.

### Part 2 Generating Correct Sound

We use previously defined structure: `cpal_stream` to write part2.

#### Tests

You can test part2 ck1 by

```bash
cargo test part2_ck1 --release -- --ignored
```

The program will play the signal
$$f(t) = \sin (2\pi \cdot 1000t) + \sin(2\pi\cdot 10000t)$$
You will get a spectrum similar to the picture:
![Spectrum](./img/spectrum.jpg)

### Part 3 Transmitting Your First Bit

In this part, we implement

- PSK modulation: `PSK` is an implementor of `Modem` which supoort `modulate` and `demodulate`.([psk.rs](./proj1_acoustic_link/src/phy_packet/modem/psk.rs))
- Correlation frame detection: `CorrelationFraming` is an implementor of `FrameDetector` which support `on_sample`.([frame_detect.rs](./proj1_acoustic_link/src/phy_packet/frame_detect.rs))
- Chirp signal as preamble: `ChirpUpDown` is an implementor of `PreambleGen`, which support `generate`, `len`, `norm` and other helper functions.([preambles.rs](./proj1_acoustic_link/src/phy_packet/preambles.rs))

### Part 4

### Part 5

### Part 6

### Tests

### Acknowledgement
