//! The buzz.
//!
//! This is in the movement spike rather than in some later audio pass because it
//! is not sound design — it is a movement instrument. In a game with no HUD, the
//! wingbeat is where speed and effort are legible: it rises as the fly works,
//! drops as it coasts, and cuts out entirely the moment it lands. That last
//! transition is the strongest single piece of feedback in the build, because
//! silence is unmistakable and costs nothing to notice.
//!
//! It is synthesised rather than sampled for one reason: a sample cannot be bent
//! continuously across the whole effort range without artefacts, and continuity
//! is the entire point. A housefly's wingbeat sits near two hundred hertz with
//! strong harmonics — well inside the range where a few oscillators sound more
//! like the real thing than a looped recording does.
//!
//! The decoder runs on the audio thread and the game runs on its own, so the two
//! parameters that change are passed through atomics and slewed toward on the
//! audio side. Writing them directly would click on every frame.

use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;
use std::sync::Arc;

use bevy::audio::{AddAudioSource, ChannelCount, Decodable, SampleRate, Source};
use bevy::math::ops;
use bevy::prelude::*;
use bevy::reflect::TypePath;

const SAMPLE_RATE: u32 = 44_100;

/// Fundamental at rest and at full effort, hertz.
///
/// A housefly beats its wings at roughly 200 Hz and, in life, modulates
/// amplitude far more than frequency — so this range is deliberately narrow and
/// sits either side of the real figure. Widening it makes effort easier to hear
/// and the insect less convincing; that trade is the dial.
const HZ_IDLE: f32 = 175.0;
const HZ_WORKING: f32 = 225.0;

/// How quickly the audio thread chases a new target, as a per-sample coefficient.
/// Slow enough to never click, fast enough that a hard turn is heard on the turn
/// and not a moment after it.
const SLEW: f32 = 0.00035;

/// Overall level. Quiet: this plays continuously for as long as the fly is alive,
/// and anything louder becomes furniture within a minute.
const LEVEL: f32 = 0.16;

/// The two numbers the game writes and the audio thread reads, as raw `f32` bits.
#[derive(Debug)]
pub struct Voice {
    frequency: AtomicU32,
    gain: AtomicU32,
}

impl Voice {
    fn new() -> Self {
        Voice {
            frequency: AtomicU32::new(HZ_IDLE.to_bits()),
            gain: AtomicU32::new(0.0f32.to_bits()),
        }
    }

    fn set(&self, frequency: f32, gain: f32) {
        self.frequency.store(frequency.to_bits(), Ordering::Relaxed);
        self.gain.store(gain.to_bits(), Ordering::Relaxed);
    }

    fn read(&self) -> (f32, f32) {
        (
            f32::from_bits(self.frequency.load(Ordering::Relaxed)),
            f32::from_bits(self.gain.load(Ordering::Relaxed)),
        )
    }
}

/// The asset. Holds only the shared voice; all the state lives in the decoder.
#[derive(Asset, TypePath)]
pub struct Wingbeat {
    voice: Arc<Voice>,
}

pub struct Buzz {
    voice: Arc<Voice>,
    /// Phase of the fundamental, 0..1.
    phase: f32,
    /// Phase of the slow amplitude wobble that keeps the tone from sounding like
    /// a test signal.
    flutter: f32,
    frequency: f32,
    gain: f32,
    /// xorshift state, for the breath of noise under the harmonics.
    noise: u32,
}

impl Buzz {
    fn new(voice: Arc<Voice>) -> Self {
        let (frequency, gain) = voice.read();
        Buzz {
            voice,
            phase: 0.0,
            flutter: 0.0,
            frequency,
            gain,
            noise: 0x9E3779B9,
        }
    }

    fn white(&mut self) -> f32 {
        // xorshift32. Deterministic and free; the audio thread cannot afford a
        // real generator and does not need one.
        self.noise ^= self.noise << 13;
        self.noise ^= self.noise >> 17;
        self.noise ^= self.noise << 5;
        (self.noise as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

impl Iterator for Buzz {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let (want_frequency, want_gain) = self.voice.read();
        self.frequency += (want_frequency - self.frequency) * SLEW;
        self.gain += (want_gain - self.gain) * SLEW;

        let step = self.frequency / SAMPLE_RATE as f32;
        self.phase = (self.phase + step).fract();
        self.flutter = (self.flutter + 7.3 / SAMPLE_RATE as f32).fract();

        let turn = core::f32::consts::TAU * self.phase;
        // Fundamental plus two harmonics. The third is what makes it read as an
        // insect rather than as a hum.
        let tone =
            ops::sin(turn) * 0.58 + ops::sin(turn * 2.0) * 0.30 + ops::sin(turn * 3.0) * 0.16;

        // A shallow wobble and a little noise. A real fly is never quite steady,
        // and a perfectly steady tone is the one thing that gives synthesis away.
        let wobble = 1.0 + 0.14 * ops::sin(core::f32::consts::TAU * self.flutter);
        let breath = self.white() * 0.05;

        Some((tone * wobble + breath) * self.gain * LEVEL)
    }
}

impl Source for Buzz {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        ChannelCount::new(1).unwrap()
    }

    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(SAMPLE_RATE).unwrap()
    }

    /// Never ends. The fly stops buzzing by going silent, not by the source
    /// running out — restarting a source every time the fly lands would click and
    /// would need the game to know when to do it.
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Decodable for Wingbeat {
    type Decoder = Buzz;

    fn decoder(&self) -> Buzz {
        Buzz::new(self.voice.clone())
    }
}

/// The handle the game writes through.
#[derive(Resource)]
struct TheVoice(Arc<Voice>);

pub struct WingbeatPlugin;

impl Plugin for WingbeatPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_source::<Wingbeat>()
            .add_systems(Startup, start_buzzing)
            .add_systems(Update, follow_the_effort);
    }
}

fn start_buzzing(mut commands: Commands, mut sources: ResMut<Assets<Wingbeat>>) {
    let voice = Arc::new(Voice::new());
    let handle = sources.add(Wingbeat {
        voice: voice.clone(),
    });
    commands.insert_resource(TheVoice(voice));
    commands.spawn((Name::new("Wingbeat"), AudioPlayer(handle)));
}

fn follow_the_effort(voice: Option<Res<TheVoice>>, flies: Query<&crate::fly::Fly>) {
    let (Some(voice), Ok(fly)) = (voice, flies.single()) else {
        return;
    };
    let effort = fly.effort();

    // Perched is silent, and the silence is the point.
    let gain = match fly.stance {
        crate::fly::Stance::Perched(_) => 0.0,
        // A floor well above zero: hovering still costs a fly everything it has,
        // and a hover that fades to nothing would say the opposite.
        crate::fly::Stance::Flying => 0.45 + 0.55 * effort.min(1.0),
    };
    let frequency = HZ_IDLE + (HZ_WORKING - HZ_IDLE) * effort.min(1.0);
    voice.0.set(frequency, gain);
}
