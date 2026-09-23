use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::world::{CellType, World};

pub struct ActiveEmitterState {
    pub sound_path: String,
    pub is_playing: bool,
    pub is_paused: bool,
    pub is_looped: bool,
    pub volume: f32,
    pub sink: Option<Sink>,
}

pub struct AudioSystem {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    emitters: HashMap<u64, ActiveEmitterState>,
}

impl AudioSystem {
    pub fn new() -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!(
                    "[AUDIO] No audio output device available ({}), running in silent mode.",
                    e
                );
                (None, None)
            }
        };

        Self {
            _stream: stream,
            stream_handle: handle,
            emitters: HashMap::new(),
        }
    }

    pub fn update(&mut self, world: &World, project_path: &Option<PathBuf>) {
        let mut active_cell_ids = Vec::new();

        for coord in world.active_effective_blocks() {
            if let Some(cell) = world.get_effective_cell(coord) {
                if cell.cell_type == CellType::AudioEmitter {
                    let id = cell.id;
                    active_cell_ids.push(id);

                    let playing = world.is_audio_playing(id);
                    println!(
                        "[AUDIO] emitter={} playing={} authored={} path={}",
                        id,
                        playing,
                        world
                            .get_effective_cell_by_id(id)
                            .map(|c| c.playing)
                            .unwrap_or(false),
                        world.get_audio_path(id),
                    );
                    let paused = world.is_audio_paused(id);
                    let looped = world.is_audio_looped(id);
                    let volume = world.get_audio_volume(id);
                    let audio_rel_path = world.get_audio_path(id);

                    if playing && !paused && !audio_rel_path.trim().is_empty() {
                        let need_start = match self.emitters.get(&id) {
                            Some(state) => {
                                !state.is_playing
                                    || state.sound_path != audio_rel_path
                                    || state.is_looped != looped
                            }
                            None => true,
                        };

                        if need_start {
                            if let Some(old_state) = self.emitters.remove(&id) {
                                if let Some(sink) = old_state.sink {
                                    sink.stop();
                                }
                            }

                            let file_path = resolve_audio_file_path(&audio_rel_path, project_path);

                            let mut sink_opt = None;
                            if let Some(ref path) = file_path {
                                if let Some(ref handle) = self.stream_handle {
                                    if let Ok(file) = File::open(path) {
                                        let reader = BufReader::new(file);
                                        if let Ok(source) = Decoder::new(reader) {
                                            if let Ok(sink) = Sink::try_new(handle) {
                                                sink.set_volume(volume);
                                                if looped {
                                                    sink.append(source.repeat_infinite());
                                                } else {
                                                    println!("[AUDIO] Playing emitter");
                                                    sink.append(source);
                                                }
                                                sink.play();
                                                sink_opt = Some(sink);
                                            }
                                        }
                                    }
                                }
                            }

                            self.emitters.insert(
                                id,
                                ActiveEmitterState {
                                    sound_path: audio_rel_path,
                                    is_playing: true,
                                    is_paused: false,
                                    is_looped: looped,
                                    volume,
                                    sink: sink_opt,
                                },
                            );
                        } else if let Some(state) = self.emitters.get_mut(&id) {
                            if (state.volume - volume).abs() > 1e-4 {
                                state.volume = volume;
                                if let Some(ref sink) = state.sink {
                                    sink.set_volume(volume);
                                }
                            }
                            if state.is_paused {
                                state.is_paused = false;
                                if let Some(ref sink) = state.sink {
                                    sink.play();
                                }
                            }
                        }
                    } else if paused && playing {
                        if let Some(state) = self.emitters.get_mut(&id) {
                            if !state.is_paused {
                                state.is_paused = true;
                                if let Some(ref sink) = state.sink {
                                    sink.pause();
                                }
                            }
                        }
                    } else {
                        if let Some(state) = self.emitters.remove(&id) {
                            if let Some(sink) = state.sink {
                                sink.stop();
                            }
                        }
                    }
                }
            }
        }

        let to_remove: Vec<u64> = self
            .emitters
            .keys()
            .cloned()
            .filter(|id| !active_cell_ids.contains(id))
            .collect();

        for id in to_remove {
            if let Some(state) = self.emitters.remove(&id) {
                if let Some(sink) = state.sink {
                    sink.stop();
                }
            }
        }
    }

    pub fn stop_all(&mut self) {
        for (_, state) in self.emitters.drain() {
            if let Some(sink) = state.sink {
                sink.stop();
            }
        }
    }
}

pub fn resolve_audio_file_path(
    rel_path: &str,
    project_path: &Option<PathBuf>,
) -> Option<PathBuf> {
    if rel_path.trim().is_empty() {
        return None;
    }

    if let Some(proj) = project_path {
        let proj_file = proj.join(".assets").join("audio").join(rel_path);
        if proj_file.exists() {
            return Some(proj_file);
        }
    }

    let builtin_file = Path::new(".assets").join("audio").join(rel_path);
    if builtin_file.exists() {
        return Some(builtin_file);
    }

    None
}
