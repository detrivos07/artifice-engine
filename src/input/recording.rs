use crate::events::core::{Event, EventData, KeyCode, KeyAction, MouseButton};
use artifice_logging::{debug, info, warn, error};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Represents a recorded input event with timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    /// Time offset from the start of the recording (in milliseconds)
    pub timestamp_ms: u64,
    /// The actual event data
    pub event_data: SerializableEventData,
}

/// Serializable version of EventData for recording purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableEventData {
    Key {
        key: u32, // KeyCode as u32 for serialization
        action: u8, // KeyAction as u8
        shift: bool,
        control: bool,
        alt: bool,
        super_key: bool,
    },
    MouseMove {
        x: f64,
        y: f64,
    },
    MouseButton {
        button: u8, // MouseButton as u8
        action: u8, // KeyAction as u8 (reused for mouse)
        shift: bool,
        control: bool,
        alt: bool,
        super_key: bool,
    },
    MouseScroll {
        x_offset: f64,
        y_offset: f64,
    },
}

impl SerializableEventData {
    /// Convert from EventData to SerializableEventData
    pub fn from_event_data(data: &EventData) -> Option<Self> {
        match data {
            EventData::Key(key_event) => Some(SerializableEventData::Key {
                key: key_event.key as u32,
                action: key_event.action as u8,
                shift: key_event.mods.shift,
                control: key_event.mods.control,
                alt: key_event.mods.alt,
                super_key: key_event.mods.super_key,
            }),
            EventData::MouseMove(move_event) => Some(SerializableEventData::MouseMove {
                x: move_event.x,
                y: move_event.y,
            }),
            EventData::MouseButton(button_event) => Some(SerializableEventData::MouseButton {
                button: button_event.button as u8,
                action: button_event.action as u8,
                shift: button_event.mods.shift,
                control: button_event.mods.control,
                alt: button_event.mods.alt,
                super_key: button_event.mods.super_key,
            }),
            EventData::MouseScroll(scroll_event) => Some(SerializableEventData::MouseScroll {
                x_offset: scroll_event.x_offset,
                y_offset: scroll_event.y_offset,
            }),
            _ => None, // Don't record window or application events
        }
    }

    /// Convert to EventData for replay
    pub fn to_event_data(&self) -> Option<EventData> {
        use crate::events::core::{KeyEvent, MouseMoveEvent, MouseButtonEvent, MouseScrollEvent, KeyAction, KeyMod, KeyCode, MouseButton};
        
        match self {
            SerializableEventData::Key { key, action, shift, control, alt, super_key } => {
                Some(EventData::Key(KeyEvent {
                    key: Self::u32_to_keycode(*key),
                    action: Self::u8_to_keyaction(*action),
                    mods: KeyMod {
                        shift: *shift,
                        control: *control,
                        alt: *alt,
                        super_key: *super_key,
                        caps_lock: false,
                        num_lock: false,
                    },
                }))
            },
            SerializableEventData::MouseMove { x, y } => {
                Some(EventData::MouseMove(MouseMoveEvent {
                    x: *x,
                    y: *y,
                }))
            },
            SerializableEventData::MouseButton { button, action, shift, control, alt, super_key } => {
                Some(EventData::MouseButton(MouseButtonEvent {
                    button: Self::u8_to_mousebutton(*button),
                    action: Self::u8_to_keyaction(*action),
                    mods: KeyMod {
                        shift: *shift,
                        control: *control,
                        alt: *alt,
                        super_key: *super_key,
                        caps_lock: false,
                        num_lock: false,
                    },
                }))
            },
            SerializableEventData::MouseScroll { x_offset, y_offset } => {
                Some(EventData::MouseScroll(MouseScrollEvent {
                    x_offset: *x_offset,
                    y_offset: *y_offset,
                }))
            },
        }
    }

    fn u32_to_keycode(value: u32) -> KeyCode {
        match value {
            0 => KeyCode::Unknown,
            1 => KeyCode::Space,
            2 => KeyCode::Apostrophe,
            3 => KeyCode::Comma,
            4 => KeyCode::Minus,
            5 => KeyCode::Period,
            6 => KeyCode::Slash,
            7 => KeyCode::Num0,
            8 => KeyCode::Num1,
            9 => KeyCode::Num2,
            10 => KeyCode::Num3,
            11 => KeyCode::Num4,
            12 => KeyCode::Num5,
            13 => KeyCode::Num6,
            14 => KeyCode::Num7,
            15 => KeyCode::Num8,
            16 => KeyCode::Num9,
            17 => KeyCode::Semicolon,
            18 => KeyCode::Equal,
            19 => KeyCode::A,
            20 => KeyCode::B,
            21 => KeyCode::C,
            22 => KeyCode::D,
            23 => KeyCode::E,
            24 => KeyCode::F,
            25 => KeyCode::G,
            26 => KeyCode::H,
            27 => KeyCode::I,
            28 => KeyCode::J,
            29 => KeyCode::K,
            30 => KeyCode::L,
            31 => KeyCode::M,
            32 => KeyCode::N,
            33 => KeyCode::O,
            34 => KeyCode::P,
            35 => KeyCode::Q,
            36 => KeyCode::R,
            37 => KeyCode::S,
            38 => KeyCode::T,
            39 => KeyCode::U,
            40 => KeyCode::V,
            41 => KeyCode::W,
            42 => KeyCode::X,
            43 => KeyCode::Y,
            44 => KeyCode::Z,
            45 => KeyCode::LeftBracket,
            46 => KeyCode::Backslash,
            47 => KeyCode::RightBracket,
            48 => KeyCode::GraveAccent,
            49 => KeyCode::World1,
            50 => KeyCode::World2,
            51 => KeyCode::Escape,
            52 => KeyCode::Enter,
            53 => KeyCode::Tab,
            54 => KeyCode::Backspace,
            55 => KeyCode::Insert,
            56 => KeyCode::Delete,
            57 => KeyCode::Right,
            58 => KeyCode::Left,
            59 => KeyCode::Down,
            60 => KeyCode::Up,
            61 => KeyCode::PageUp,
            62 => KeyCode::PageDown,
            63 => KeyCode::Home,
            64 => KeyCode::End,
            65 => KeyCode::CapsLock,
            66 => KeyCode::ScrollLock,
            67 => KeyCode::NumLock,
            68 => KeyCode::PrintScreen,
            69 => KeyCode::Pause,
            70 => KeyCode::F1,
            71 => KeyCode::F2,
            72 => KeyCode::F3,
            73 => KeyCode::F4,
            74 => KeyCode::F5,
            75 => KeyCode::F6,
            76 => KeyCode::F7,
            77 => KeyCode::F8,
            78 => KeyCode::F9,
            79 => KeyCode::F10,
            80 => KeyCode::F11,
            81 => KeyCode::F12,
            82 => KeyCode::F13,
            83 => KeyCode::F14,
            84 => KeyCode::F15,
            85 => KeyCode::F16,
            86 => KeyCode::F17,
            87 => KeyCode::F18,
            88 => KeyCode::F19,
            89 => KeyCode::F20,
            90 => KeyCode::F21,
            91 => KeyCode::F22,
            92 => KeyCode::F23,
            93 => KeyCode::F24,
            94 => KeyCode::F25,
            95 => KeyCode::KP0,
            96 => KeyCode::KP1,
            97 => KeyCode::KP2,
            98 => KeyCode::KP3,
            99 => KeyCode::KP4,
            100 => KeyCode::KP5,
            101 => KeyCode::KP6,
            102 => KeyCode::KP7,
            103 => KeyCode::KP8,
            104 => KeyCode::KP9,
            105 => KeyCode::KPDecimal,
            106 => KeyCode::KPDivide,
            107 => KeyCode::KPMultiply,
            108 => KeyCode::KPSubtract,
            109 => KeyCode::KPAdd,
            110 => KeyCode::KPEnter,
            111 => KeyCode::KPEqual,
            112 => KeyCode::LeftShift,
            113 => KeyCode::LeftControl,
            114 => KeyCode::LeftAlt,
            115 => KeyCode::LeftSuper,
            116 => KeyCode::RightShift,
            117 => KeyCode::RightControl,
            118 => KeyCode::RightAlt,
            119 => KeyCode::RightSuper,
            120 => KeyCode::Menu,
            _ => KeyCode::Unknown,
        }
    }

    fn u8_to_keyaction(value: u8) -> KeyAction {
        match value {
            0 => KeyAction::Press,
            1 => KeyAction::Release,
            2 => KeyAction::Repeat,
            _ => KeyAction::Press,
        }
    }

    fn u8_to_mousebutton(value: u8) -> MouseButton {
        match value {
            0 => MouseButton::Button1,
            1 => MouseButton::Button2,
            2 => MouseButton::Button3,
            3 => MouseButton::Button4,
            4 => MouseButton::Button5,
            5 => MouseButton::Button6,
            6 => MouseButton::Button7,
            7 => MouseButton::Button8,
            8 => MouseButton::Left,
            9 => MouseButton::Right,
            10 => MouseButton::Middle,
            _ => MouseButton::Button1,
        }
    }
}

/// A complete input recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRecording {
    /// Metadata about the recording
    pub metadata: RecordingMetadata,
    /// The recorded events
    pub events: Vec<RecordedEvent>,
}

/// Metadata about an input recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// Name of the recording
    pub name: String,
    /// Description of the recording
    pub description: Option<String>,
    /// When the recording was created
    pub created_at: String,
    /// Duration of the recording in milliseconds
    pub duration_ms: u64,
    /// Number of events in the recording
    pub event_count: usize,
    /// Version of the recording format
    pub format_version: u32,
}

impl InputRecording {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            metadata: RecordingMetadata {
                name: name.into(),
                description: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                duration_ms: 0,
                event_count: 0,
                format_version: 1,
            },
            events: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = Some(description.into());
        self
    }

    /// Save the recording to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Load a recording from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let recording: InputRecording = serde_json::from_reader(reader)?;
        Ok(recording)
    }

    /// Get the duration of the recording
    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.metadata.duration_ms)
    }

    /// Update metadata after recording is complete
    fn finalize_metadata(&mut self) {
        self.metadata.event_count = self.events.len();
        if let Some(last_event) = self.events.last() {
            self.metadata.duration_ms = last_event.timestamp_ms;
        }
    }
}

/// Records input events for later playback
pub struct InputRecorder {
    recording: InputRecording,
    start_time: Option<Instant>,
    is_recording: bool,
}

impl InputRecorder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            recording: InputRecording::new(name),
            start_time: None,
            is_recording: false,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.recording = self.recording.with_description(description);
        self
    }

    /// Start recording input events
    pub fn start_recording(&mut self) {
        if self.is_recording {
            warn!("Already recording, ignoring start_recording call");
            return;
        }

        self.start_time = Some(Instant::now());
        self.is_recording = true;
        self.recording.events.clear();
        info!("Started recording input: {}", self.recording.metadata.name);
    }

    /// Stop recording and finalize the recording
    pub fn stop_recording(&mut self) {
        if !self.is_recording {
            warn!("Not currently recording, ignoring stop_recording call");
            return;
        }

        self.is_recording = false;
        self.recording.finalize_metadata();
        info!(
            "Stopped recording input: {} ({} events, {:.2}s)",
            self.recording.metadata.name,
            self.recording.metadata.event_count,
            self.recording.metadata.duration_ms as f64 / 1000.0
        );
    }

    /// Record an input event
    pub fn record_event(&mut self, event: &Event) {
        if !self.is_recording {
            return;
        }

        let start_time = match self.start_time {
            Some(time) => time,
            None => {
                warn!("Recording without start time, ignoring event");
                return;
            }
        };

        if let Some(serializable_data) = SerializableEventData::from_event_data(&event.data) {
            let timestamp_ms = start_time.elapsed().as_millis() as u64;
            
            let recorded_event = RecordedEvent {
                timestamp_ms,
                event_data: serializable_data,
            };

            self.recording.events.push(recorded_event);
        }
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Get the current recording (consumes the recorder)
    pub fn finish(mut self) -> InputRecording {
        if self.is_recording {
            self.stop_recording();
        }
        self.recording
    }

    /// Get a reference to the current recording
    pub fn get_recording(&self) -> &InputRecording {
        &self.recording
    }
}

/// Replays recorded input events
pub struct InputPlayer {
    recording: InputRecording,
    current_event_index: usize,
    playback_start_time: Option<Instant>,
    is_playing: bool,
    playback_speed: f64,
    loop_playback: bool,
}

impl InputPlayer {
    pub fn new(recording: InputRecording) -> Self {
        Self {
            recording,
            current_event_index: 0,
            playback_start_time: None,
            is_playing: false,
            playback_speed: 1.0,
            loop_playback: false,
        }
    }

    /// Load a player from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let recording = InputRecording::load_from_file(path)?;
        Ok(Self::new(recording))
    }

    /// Set the playback speed (1.0 = normal speed, 2.0 = double speed, 0.5 = half speed)
    pub fn set_playback_speed(&mut self, speed: f64) {
        self.playback_speed = speed.max(0.1).min(10.0); // Clamp between 0.1x and 10x
    }

    /// Enable or disable loop playback
    pub fn set_loop_playback(&mut self, should_loop: bool) {
        self.loop_playback = should_loop;
    }

    /// Start playback
    pub fn start_playback(&mut self) {
        if self.is_playing {
            warn!("Already playing, ignoring start_playback call");
            return;
        }

        self.playback_start_time = Some(Instant::now());
        self.current_event_index = 0;
        self.is_playing = true;
        info!(
            "Started playback: {} ({} events, {:.2}s, {:.1}x speed)",
            self.recording.metadata.name,
            self.recording.metadata.event_count,
            self.recording.metadata.duration_ms as f64 / 1000.0,
            self.playback_speed
        );
    }

    /// Stop playback
    pub fn stop_playback(&mut self) {
        if !self.is_playing {
            return;
        }

        self.is_playing = false;
        info!("Stopped playback: {}", self.recording.metadata.name);
    }

    /// Pause/resume playback
    pub fn toggle_pause(&mut self) {
        if self.is_playing {
            self.stop_playback();
        } else {
            self.start_playback();
        }
    }

    /// Reset playback to the beginning
    pub fn reset(&mut self) {
        self.current_event_index = 0;
        self.playback_start_time = None;
    }

    /// Get events that should be played at the current time
    pub fn get_current_events(&mut self) -> Vec<Event> {
        if !self.is_playing {
            return Vec::new();
        }

        let start_time = match self.playback_start_time {
            Some(time) => time,
            None => return Vec::new(),
        };

        let current_time_ms = (start_time.elapsed().as_millis() as f64 * self.playback_speed) as u64;
        let mut events = Vec::new();

        // Collect all events that should be played by now
        while self.current_event_index < self.recording.events.len() {
            let recorded_event = &self.recording.events[self.current_event_index];
            
            if recorded_event.timestamp_ms <= current_time_ms {
                if let Some(event_data) = recorded_event.event_data.to_event_data() {
                    let event = Event::new(event_data);
                    events.push(event);
                }
                self.current_event_index += 1;
            } else {
                break;
            }
        }

        // Check if playback is complete
        if self.current_event_index >= self.recording.events.len() {
            if self.loop_playback {
                debug!("Looping playback: {}", self.recording.metadata.name);
                self.reset();
                self.start_playback();
            } else {
                self.stop_playback();
            }
        }

        events
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Get playback progress (0.0 to 1.0)
    pub fn get_progress(&self) -> f64 {
        if self.recording.events.is_empty() {
            return 1.0;
        }
        self.current_event_index as f64 / self.recording.events.len() as f64
    }

    /// Get the recording metadata
    pub fn get_metadata(&self) -> &RecordingMetadata {
        &self.recording.metadata
    }

    /// Seek to a specific position in the recording (0.0 to 1.0)
    pub fn seek(&mut self, position: f64) {
        let position = position.clamp(0.0, 1.0);
        self.current_event_index = (position * self.recording.events.len() as f64) as usize;
        
        if self.is_playing {
            // Adjust start time to account for the seek
            let target_time_ms = if let Some(event) = self.recording.events.get(self.current_event_index) {
                event.timestamp_ms
            } else {
                self.recording.metadata.duration_ms
            };
            
            let adjusted_time_ms = (target_time_ms as f64 / self.playback_speed) as u64;
            self.playback_start_time = Some(Instant::now() - Duration::from_millis(adjusted_time_ms));
        }
    }
}

/// Manages multiple recordings and players
pub struct InputRecordingManager {
    recordings: Vec<InputRecording>,
    active_recorder: Option<InputRecorder>,
    active_players: Vec<InputPlayer>,
}

impl InputRecordingManager {
    pub fn new() -> Self {
        Self {
            recordings: Vec::new(),
            active_recorder: None,
            active_players: Vec::new(),
        }
    }

    /// Start recording with a new recorder
    pub fn start_recording(&mut self, name: impl Into<String>) {
        if self.active_recorder.is_some() {
            warn!("Already recording, stopping previous recording");
            self.stop_recording();
        }

        let mut recorder = InputRecorder::new(name);
        recorder.start_recording();
        self.active_recorder = Some(recorder);
    }

    /// Stop the current recording
    pub fn stop_recording(&mut self) -> Option<InputRecording> {
        if let Some(mut recorder) = self.active_recorder.take() {
            let recording = recorder.finish();
            self.recordings.push(recording.clone());
            Some(recording)
        } else {
            None
        }
    }

    /// Record an event with the active recorder
    pub fn record_event(&mut self, event: &Event) {
        if let Some(recorder) = &mut self.active_recorder {
            recorder.record_event(event);
        }
    }

    /// Add a recording to the manager
    pub fn add_recording(&mut self, recording: InputRecording) {
        self.recordings.push(recording);
    }

    /// Start playback of a recording by name
    pub fn start_playback(&mut self, name: &str) -> bool {
        if let Some(recording) = self.recordings.iter().find(|r| r.metadata.name == name).cloned() {
            let mut player = InputPlayer::new(recording);
            player.start_playback();
            self.active_players.push(player);
            true
        } else {
            false
        }
    }

    /// Stop all active playback
    pub fn stop_all_playback(&mut self) {
        for player in &mut self.active_players {
            player.stop_playback();
        }
        self.active_players.clear();
    }

    /// Get events from all active players
    pub fn get_playback_events(&mut self) -> Vec<Event> {
        let mut all_events = Vec::new();
        
        self.active_players.retain_mut(|player| {
            let events = player.get_current_events();
            all_events.extend(events);
            player.is_playing() // Keep only playing players
        });
        
        all_events
    }

    /// Get a list of all recording names
    pub fn get_recording_names(&self) -> Vec<&str> {
        self.recordings.iter().map(|r| r.metadata.name.as_str()).collect()
    }

    /// Load a recording from file and add it to the manager
    pub fn load_recording_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let recording = InputRecording::load_from_file(path)?;
        self.add_recording(recording);
        Ok(())
    }

    /// Save a recording to file by name
    pub fn save_recording_to_file<P: AsRef<Path>>(&self, name: &str, path: P) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(recording) = self.recordings.iter().find(|r| r.metadata.name == name) {
            recording.save_to_file(path)?;
            Ok(())
        } else {
            Err(format!("Recording '{}' not found", name).into())
        }
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.active_recorder.as_ref().map_or(false, |r| r.is_recording())
    }

    /// Get the number of active players
    pub fn active_player_count(&self) -> usize {
        self.active_players.len()
    }

    /// Get the number of stored recordings
    pub fn recording_count(&self) -> usize {
        self.recordings.len()
    }
}

impl Default for InputRecordingManager {
    fn default() -> Self {
        Self::new()
    }
}