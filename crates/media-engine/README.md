# Media Engine

Audio playback engine for StoryStream.

## Quick Start
```rust
use media_engine::prelude::*;

let mut engine = MediaEngine::new()?;
engine.load("audiobook.mp3", Some(1))?;
engine.play()?;