use super::asset_search::ranked_asset_matches;
use egui::Ui;
use std::fs;
use std::path::{Path, PathBuf};

const AUDIO_BROWSER_ID: &str = "aeoengine_audio_browser";
const AUDIO_CATALOG_ID: &str = "aeoengine_audio_catalog";

#[derive(Clone, Default)]
struct AudioCatalog {
    names: Vec<String>,
}

#[derive(Clone, Default)]
struct AudioBrowserState {
    open: bool,
    search: String,
}

fn scan_directory_wavs(current_dir: &Path, root_dir: &Path, names: &mut Vec<String>) {
    let entries = match fs::read_dir(current_dir) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!(
                "[EDITOR][AUDIO] Failed to read '{}': {}",
                current_dir.display(),
                error
            );
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_directory_wavs(&path, root_dir, names);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let is_wav = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("wav"))
            .unwrap_or(false);

        if !is_wav {
            continue;
        }

        if let Ok(relative_path) = path.strip_prefix(root_dir) {
            names.push(relative_path.to_string_lossy().replace('\\', "/"));
        }
    }
}

/// Returns relative paths within the audio asset roots.
///
/// Project assets are scanned in addition to built-in assets. The project
/// audio root is checked first by the runtime, so duplicate relative paths
/// resolve to the project asset.
fn scan_audio_names(project_path: &Option<PathBuf>) -> Vec<String> {
    let mut names = Vec::new();

    let builtin_dir = PathBuf::from(".assets/audio");

    if builtin_dir.exists() {
        scan_directory_wavs(&builtin_dir, &builtin_dir, &mut names);
    }

    if let Some(project) = project_path {
        let project_dir = project.join(".assets").join("audio");

        if project_dir.exists() {
            scan_directory_wavs(&project_dir, &project_dir, &mut names);
        }
    }

    names.sort_unstable();
    names.dedup();
    names
}

fn load_audio_catalog(project_path: &Option<PathBuf>) -> AudioCatalog {
    AudioCatalog {
        names: scan_audio_names(project_path),
    }
}

fn get_audio_catalog(ctx: &egui::Context, project_path: &Option<PathBuf>) -> AudioCatalog {
    let id = egui::Id::new(AUDIO_CATALOG_ID);

    if let Some(catalog) = ctx.data(|data| data.get_temp::<AudioCatalog>(id)) {
        return catalog;
    }

    let catalog = load_audio_catalog(project_path);

    ctx.data_mut(|data| {
        data.insert_temp(id, catalog.clone());
    });

    catalog
}

fn refresh_audio_catalog(ctx: &egui::Context, project_path: &Option<PathBuf>) {
    let catalog = load_audio_catalog(project_path);

    ctx.data_mut(|data| {
        data.insert_temp(egui::Id::new(AUDIO_CATALOG_ID), catalog);
    });
}

fn open_audio_browser(ctx: &egui::Context, project_path: &Option<PathBuf>) {
    refresh_audio_catalog(ctx, project_path);

    let id = egui::Id::new(AUDIO_BROWSER_ID);

    ctx.data_mut(|data| {
        let mut state = data.get_temp::<AudioBrowserState>(id).unwrap_or_default();

        state.open = true;

        data.insert_temp(id, state);
    });
}

fn audio_matches(catalog: &AudioCatalog, audio: &str) -> Vec<String> {
    ranked_asset_matches(&catalog.names, audio, 6)
}

/// Draw the audio asset controls.
pub fn draw_audio_edit(ui: &mut Ui, audio: &mut String, project_path: &Option<PathBuf>) {
    let catalog = get_audio_catalog(ui.ctx(), project_path);

    ui.horizontal(|ui| {
        ui.label("Audio:");

        ui.add(
            egui::TextEdit::singleline(audio)
                .desired_width(140.0)
                .hint_text("Audio asset path"),
        );

        if ui.button("Browse...").clicked() {
            open_audio_browser(ui.ctx(), project_path);
        }

        if ui.button("Import...").clicked() {
            import_audio(ui.ctx(), audio, project_path);
        }
    });

    if audio.trim().is_empty() {
        return;
    }

    let matches = audio_matches(&catalog, audio);

    if matches.is_empty() {
        return;
    }

    ui.add_space(3.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(140.0);

        for match_name in matches {
            let selected = match_name == *audio;

            if ui
                .add(egui::SelectableLabel::new(selected, match_name.clone()))
                .clicked()
            {
                *audio = match_name;
            }
        }
    });
}

/// Import a WAV file into the active project's audio directory.
///
/// When no project is active, the file is imported into the built-in
/// `.assets/audio` directory.
pub fn import_audio(ctx: &egui::Context, audio: &mut String, project_path: &Option<PathBuf>) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("WAV Audio", &["wav"])
        .pick_file()
    else {
        return;
    };

    let destination_dir = project_path
        .as_ref()
        .map(|project| project.join(".assets").join("audio"))
        .unwrap_or_else(|| PathBuf::from(".assets/audio"));

    if let Err(error) = fs::create_dir_all(&destination_dir) {
        eprintln!(
            "[EDITOR][AUDIO] Failed to create '{}': {}",
            destination_dir.display(),
            error
        );
        return;
    }

    let Some(file_name) = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
    else {
        eprintln!("[EDITOR][AUDIO] Imported file has no filename.");
        return;
    };

    let stem = path
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "audio".to_string());

    let mut destination = destination_dir.join(&file_name);

    if destination.exists() {
        let mut index = 1;

        loop {
            let candidate = destination_dir.join(format!("{}_{}.wav", stem, index));

            if !candidate.exists() {
                destination = candidate;
                break;
            }

            index += 1;
        }
    }

    match fs::copy(&path, &destination) {
        Ok(_) => {
            if let Ok(relative_path) = destination.strip_prefix(&destination_dir) {
                *audio = relative_path.to_string_lossy().replace('\\', "/");
            } else if let Some(name) = destination.file_name() {
                *audio = name.to_string_lossy().to_string();
            }

            refresh_audio_catalog(ctx, project_path);
        }

        Err(error) => {
            eprintln!(
                "[EDITOR][AUDIO] Failed to copy '{}' -> '{}': {}",
                path.display(),
                destination.display(),
                error
            );
        }
    }
}

pub(crate) fn draw_audio_browser(
    ctx: &egui::Context,
    selected_audio: &mut String,
    project_path: &Option<PathBuf>,
) {
    let id = egui::Id::new(AUDIO_BROWSER_ID);

    let mut state = ctx
        .data(|data| data.get_temp::<AudioBrowserState>(id))
        .unwrap_or_default();

    if !state.open {
        return;
    }

    let mut open = state.open;
    let mut chosen_audio_name = None;

    egui::Window::new("Audio Browser")
        .open(&mut open)
        .default_size(egui::vec2(600.0, 400.0))
        .min_size(egui::vec2(360.0, 280.0))
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Search:");

                ui.add(
                    egui::TextEdit::singleline(&mut state.search)
                        .desired_width(240.0)
                        .hint_text("Filter audio assets..."),
                );

                if ui.button("Refresh").clicked() {
                    refresh_audio_catalog(ctx, project_path);
                }
            });

            ui.separator();

            let catalog = get_audio_catalog(ctx, project_path);

            let search = state.search.trim().to_lowercase();

            let names: Vec<&String> = catalog
                .names
                .iter()
                .filter(|name| search.is_empty() || name.to_lowercase().contains(&search))
                .collect();

            if names.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);

                    if catalog.names.is_empty() {
                        ui.label("No WAV audio assets found.");
                    } else {
                        ui.label("No audio assets match the search.");
                    }
                });

                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for name in names {
                        let selected = selected_audio.as_str() == name.as_str();

                        ui.horizontal(|ui| {
                            ui.label("🎵");

                            if ui.selectable_label(selected, name).clicked() {
                                chosen_audio_name = Some(name.clone());
                            }
                        });
                    }
                });
        });

    if let Some(name) = chosen_audio_name {
        *selected_audio = name;
        open = false;
    }

    state.open = open;

    ctx.data_mut(|data| {
        data.insert_temp(id, state);
    });
}
