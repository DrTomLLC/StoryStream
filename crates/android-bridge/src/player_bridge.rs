// Player control bridge for Android JNI
//
// This module provides JNI bindings for audio playback control including
// play, pause, seek, and state management.

use crate::ffi::{bool_to_jboolean, jstring_raw_to_string, FfiError, FfiResult, HandleManager};
use jni::{
    objects::JClass,
    sys::{jboolean, jdouble, jint, jlong, jstring},
    JNIEnv,
};
use once_cell::sync::Lazy;
use std::panic; // Required for jni_safe! macro
use std::sync::{Arc, RwLock};
use std::time::Duration;

// Import audio player from media-engine if available
#[cfg(feature = "media-engine")]
use storystream_media_engine::AudioPlayer;

// Mock AudioPlayer for when media-engine isn't available
#[cfg(not(feature = "media-engine"))]
pub struct AudioPlayer {
    position: Arc<RwLock<f64>>,
    duration: Arc<RwLock<f64>>,
    is_playing: Arc<RwLock<bool>>,
    speed: Arc<RwLock<f64>>,
    volume: Arc<RwLock<f64>>,
}

#[cfg(not(feature = "media-engine"))]
impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            position: Arc::new(RwLock::new(0.0)),
            duration: Arc::new(RwLock::new(0.0)),
            is_playing: Arc::new(RwLock::new(false)),
            speed: Arc::new(RwLock::new(1.0)),
            volume: Arc::new(RwLock::new(1.0)),
        }
    }

    pub fn play(&self) -> Result<(), String> {
        *self.is_playing.write().unwrap() = true;
        Ok(())
    }

    pub fn pause(&self) -> Result<(), String> {
        *self.is_playing.write().unwrap() = false;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        *self.is_playing.write().unwrap() = false;
        *self.position.write().unwrap() = 0.0;
        Ok(())
    }

    pub fn seek(&self, position: Duration) -> Result<(), String> {
        *self.position.write().unwrap() = position.as_secs_f64();
        Ok(())
    }

    pub fn position(&self) -> Duration {
        Duration::from_secs_f64(*self.position.read().unwrap())
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(*self.duration.read().unwrap())
    }

    pub fn is_playing(&self) -> bool {
        *self.is_playing.read().unwrap()
    }

    pub fn set_speed(&self, speed: f64) -> Result<(), String> {
        if !(0.25..=4.0).contains(&speed) {
            return Err("Speed must be between 0.25 and 4.0".to_string());
        }
        *self.speed.write().unwrap() = speed;
        Ok(())
    }

    pub fn speed(&self) -> f64 {
        *self.speed.read().unwrap()
    }

    pub fn set_volume(&self, volume: f64) -> Result<(), String> {
        if !(0.0..=1.0).contains(&volume) {
            return Err("Volume must be between 0.0 and 1.0".to_string());
        }
        *self.volume.write().unwrap() = volume;
        Ok(())
    }

    pub fn volume(&self) -> f64 {
        *self.volume.read().unwrap()
    }

    pub fn load(&self, _path: &str) -> Result<(), String> {
        *self.duration.write().unwrap() = 3600.0; // Mock 1 hour duration
        Ok(())
    }
}

/// Global player handle manager
static PLAYER_HANDLES: Lazy<HandleManager<Arc<AudioPlayer>>> = Lazy::new(HandleManager::new);

/// Create a new player instance
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeCreate(
    mut env: JNIEnv,
    _class: JClass,
) -> jlong {
    crate::jni_safe!(env, 0, {
        let player = Arc::new(AudioPlayer::new());
        let handle = PLAYER_HANDLES.insert(player);

        crate::ffi::log_info("StoryStream", &format!("Created player handle: {}", handle));

        Ok(handle)
    })
}

/// Load an audio file for playback
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeLoad(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    audio_file_path: jstring,
) -> jboolean {
    crate::jni_safe!(env, bool_to_jboolean(false), {
        let player = PLAYER_HANDLES.get(handle)?;
        let path = jstring_raw_to_string(&mut env, audio_file_path)?;

        crate::ffi::log_info("StoryStream", &format!("Loading audio file: {}", path));

        player
            .read()
            .unwrap()
            .load(&path)
            .map_err(|e| FfiError::General(format!("Failed to load audio: {}", e)))?;

        Ok(bool_to_jboolean(true))
    })
}

/// Start or resume playback
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativePlay(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    crate::jni_safe!(env, bool_to_jboolean(false), {
        let player = PLAYER_HANDLES.get(handle)?;

        crate::ffi::log_info("StoryStream", "Starting playback");

        player
            .read()
            .unwrap()
            .play()
            .map_err(|e| FfiError::General(format!("Failed to play: {}", e)))?;

        Ok(bool_to_jboolean(true))
    })
}

/// Pause playback
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativePause(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    crate::jni_safe!(env, bool_to_jboolean(false), {
        let player = PLAYER_HANDLES.get(handle)?;

        crate::ffi::log_info("StoryStream", "Pausing playback");

        player
            .read()
            .unwrap()
            .pause()
            .map_err(|e| FfiError::General(format!("Failed to pause: {}", e)))?;

        Ok(bool_to_jboolean(true))
    })
}

/// Stop playback and reset position
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeStop(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    crate::jni_safe!(env, bool_to_jboolean(false), {
        let player = PLAYER_HANDLES.get(handle)?;

        crate::ffi::log_info("StoryStream", "Stopping playback");

        player
            .read()
            .unwrap()
            .stop()
            .map_err(|e| FfiError::General(format!("Failed to stop: {}", e)))?;

        Ok(bool_to_jboolean(true))
    })
}

/// Seek to position in seconds
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeSeek(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    position_seconds: jdouble,
) -> jboolean {
    crate::jni_safe!(env, bool_to_jboolean(false), {
        let player = PLAYER_HANDLES.get(handle)?;

        if position_seconds < 0.0 {
            return Err(FfiError::General("Position cannot be negative".to_string()));
        }

        crate::ffi::log_info(
            "StoryStream",
            &format!("Seeking to: {:.2}s", position_seconds),
        );

        player
            .read()
            .unwrap()
            .seek(Duration::from_secs_f64(position_seconds))
            .map_err(|e| FfiError::General(format!("Failed to seek: {}", e)))?;

        Ok(bool_to_jboolean(true))
    })
}

/// Get current playback position in seconds
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeGetPosition(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jdouble {
    crate::jni_safe!(env, 0.0, {
        let player = PLAYER_HANDLES.get(handle)?;
        let position = player.read().unwrap().position();
        Ok(position.as_secs_f64())
    })
}

/// Get total duration in seconds
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeGetDuration(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jdouble {
    crate::jni_safe!(env, 0.0, {
        let player = PLAYER_HANDLES.get(handle)?;
        let duration = player.read().unwrap().duration();
        Ok(duration.as_secs_f64())
    })
}

/// Check if currently playing
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeIsPlaying(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    crate::jni_safe!(env, bool_to_jboolean(false), {
        let player = PLAYER_HANDLES.get(handle)?;
        let is_playing = player.read().unwrap().is_playing();
        Ok(bool_to_jboolean(is_playing))
    })
}

/// Set playback speed (0.25 to 4.0, 1.0 = normal)
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeSetSpeed(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    speed: jdouble,
) -> jboolean {
    crate::jni_safe!(env, bool_to_jboolean(false), {
        let player = PLAYER_HANDLES.get(handle)?;

        if !(0.25..=4.0).contains(&speed) {
            return Err(FfiError::General(
                "Speed must be between 0.25 and 4.0".to_string(),
            ));
        }

        crate::ffi::log_info("StoryStream", &format!("Setting speed: {:.2}x", speed));

        player
            .read()
            .unwrap()
            .set_speed(speed)
            .map_err(|e| FfiError::General(format!("Failed to set speed: {}", e)))?;

        Ok(bool_to_jboolean(true))
    })
}

/// Get current playback speed
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeGetSpeed(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jdouble {
    crate::jni_safe!(env, 1.0, {
        let player = PLAYER_HANDLES.get(handle)?;
        let speed = player.read().unwrap().speed();
        Ok(speed)
    })
}

/// Set volume (0.0 to 1.0)
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeSetVolume(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    volume: jdouble,
) -> jboolean {
    crate::jni_safe!(env, bool_to_jboolean(false), {
        let player = PLAYER_HANDLES.get(handle)?;

        if !(0.0..=1.0).contains(&volume) {
            return Err(FfiError::General(
                "Volume must be between 0.0 and 1.0".to_string(),
            ));
        }

        crate::ffi::log_info("StoryStream", &format!("Setting volume: {:.2}", volume));

        player
            .read()
            .unwrap()
            .set_volume(volume)
            .map_err(|e| FfiError::General(format!("Failed to set volume: {}", e)))?;

        Ok(bool_to_jboolean(true))
    })
}

/// Get current volume
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeGetVolume(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jdouble {
    crate::jni_safe!(env, 1.0, {
        let player = PLAYER_HANDLES.get(handle)?;
        let volume = player.read().unwrap().volume();
        Ok(volume)
    })
}

/// Get current chapter index
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeGetCurrentChapter(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    crate::jni_safe!(env, -1, {
        let _player = PLAYER_HANDLES.get(handle)?;

        // Placeholder: Would return actual chapter index
        Ok(0)
    })
}

/// Skip to specific chapter
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeSkipToChapter(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    chapter_index: jint,
) -> jboolean {
    crate::jni_safe!(env, bool_to_jboolean(false), {
        let _player = PLAYER_HANDLES.get(handle)?;

        if chapter_index < 0 {
            return Err(FfiError::General(
                "Chapter index cannot be negative".to_string(),
            ));
        }

        crate::ffi::log_info(
            "StoryStream",
            &format!("Skipping to chapter: {}", chapter_index),
        );

        // Placeholder: Would skip to chapter
        Ok(bool_to_jboolean(true))
    })
}

/// Destroy player instance
#[no_mangle]
pub extern "C" fn Java_com_storystream_StoryStreamPlayer_nativeDestroy(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    crate::jni_safe!(env, (), {
        PLAYER_HANDLES.remove(handle)?;
        crate::ffi::log_info(
            "StoryStream",
            &format!("Destroyed player handle: {}", handle),
        );
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_player_creation() {
        let player = AudioPlayer::new();
        assert!(!player.is_playing());
        assert_eq!(player.volume(), 1.0);
        assert_eq!(player.speed(), 1.0);
    }

    #[test]
    fn test_player_handle_lifecycle() {
        let player = Arc::new(AudioPlayer::new());
        let handle = PLAYER_HANDLES.insert(player);
        assert!(handle > 0);

        assert!(PLAYER_HANDLES.contains(handle));
        assert!(PLAYER_HANDLES.get(handle).is_ok());

        let removed = PLAYER_HANDLES.remove(handle);
        assert!(removed.is_ok());

        assert!(!PLAYER_HANDLES.contains(handle));
        assert!(PLAYER_HANDLES.get(handle).is_err());
    }

    #[test]
    fn test_multiple_players() {
        let player1 = Arc::new(AudioPlayer::new());
        let player2 = Arc::new(AudioPlayer::new());

        let handle1 = PLAYER_HANDLES.insert(player1);
        let handle2 = PLAYER_HANDLES.insert(player2);

        assert_ne!(handle1, handle2);
        assert!(PLAYER_HANDLES.contains(handle1));
        assert!(PLAYER_HANDLES.contains(handle2));

        PLAYER_HANDLES.remove(handle1).unwrap();

        assert!(!PLAYER_HANDLES.contains(handle1));
        assert!(PLAYER_HANDLES.contains(handle2));

        PLAYER_HANDLES.remove(handle2).unwrap();
    }

    #[test]
    fn test_player_operations() {
        let player = AudioPlayer::new();

        // Test play/pause
        assert!(player.play().is_ok());
        assert!(player.is_playing());

        assert!(player.pause().is_ok());
        assert!(!player.is_playing());

        // Test stop
        assert!(player.stop().is_ok());
        assert!(!player.is_playing());
        assert_eq!(player.position().as_secs_f64(), 0.0);

        // Test seek
        assert!(player.seek(Duration::from_secs(30)).is_ok());
        assert_eq!(player.position().as_secs(), 30);

        // Test speed
        assert!(player.set_speed(1.5).is_ok());
        assert_eq!(player.speed(), 1.5);

        assert!(player.set_speed(0.1).is_err()); // Too slow
        assert!(player.set_speed(5.0).is_err()); // Too fast

        // Test volume
        assert!(player.set_volume(0.5).is_ok());
        assert_eq!(player.volume(), 0.5);

        assert!(player.set_volume(-0.1).is_err()); // Negative
        assert!(player.set_volume(1.1).is_err()); // Too loud
    }
}
