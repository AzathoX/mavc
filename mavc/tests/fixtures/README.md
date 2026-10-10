# Synthetic audio fixtures

These tiny silence recordings were constructed locally for tests; they contain no downloaded music.

- `silence.mp3`: ten MPEG-1 Layer III frames, stereo, 44.1 kHz, 128 kbit/s; zero-filled silent frame data.
- `silence.aac`: ten AAC-LC stereo ADTS frames at 44.1 kHz, each with a silent coded payload.
- `silence.flac`: one constant mono 16-bit block of sixteen samples at 44.1 kHz, with STREAMINFO sample count, PCM MD5, and frame/header CRCs.

Integration tests decode all three through the production import validator and verify lossless archive extraction. Corruption cases modify copies of these fixtures.

- `silence.wav`: locally generated stereo 16-bit PCM, 44.1 kHz, 1024 silent frames.
- `silence.ogg`: locally assembled mono Vorbis silence with valid Ogg page CRCs and EOS. Identification/setup codebooks were taken from the local `cargo-mobile2` 0.22.5 Bevy demo fixture (`assets/audio/flying.ogg`); the comment header and all audio packets were generated locally. No music/audio packets were copied.
