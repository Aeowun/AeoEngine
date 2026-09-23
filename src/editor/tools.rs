use super::editor::{ColorTarget, Editor, EditorTool};
use crate::engine::EditorMode;
use egui::{Color32, RichText, Ui};
use glam::Vec3;
use std::fs;
use std::path::{Path, PathBuf};

// -----------------------------------------------------------------------------
// Texture browser
// -----------------------------------------------------------------------------

const TEXTURE_BROWSER_ID: &str = "aeoengine_texture_browser";
const TEXTURE_CATALOG_ID: &str = "aeoengine_texture_catalog";

const TEXTURE_PREVIEW_SIZE: f32 = 88.0;
const TEXTURE_TILE_WIDTH: f32 = 112.0;
const TEXTURE_TILE_SPACING: f32 = 10.0;

#[derive(Clone)]
struct TexturePreview {
    name: String,
    texture: egui::TextureHandle,
}

#[derive(Clone, Default)]
struct TextureCatalog {
    names: Vec<String>,
    previews: Vec<TexturePreview>,
}

#[derive(Clone, Default)]
struct TextureBrowserState {
    open: bool,
    search: String,
}

fn texture_directory() -> PathBuf {
    PathBuf::from(".assets/textures")
}

fn ensure_texture_directory() -> Option<PathBuf> {
    let path = texture_directory();

    if !path.exists() {
        if let Err(error) = fs::create_dir_all(&path) {
            eprintln!(
                "[EDITOR][TEXTURES] Failed to create '{}': {}",
                path.display(),
                error
            );
            return None;
        }
    }

    Some(path)
}

fn scan_texture_names() -> Vec<String> {
    let Some(directory) = ensure_texture_directory() else {
        return Vec::new();
    };

    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!(
                "[EDITOR][TEXTURES] Failed to read '{}': {}",
                directory.display(),
                error
            );
            return Vec::new();
        }
    };

    let mut names = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        let is_png = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("png"))
            .unwrap_or(false);

        if !is_png {
            continue;
        }

        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            names.push(stem.to_string());
        }
    }

    names.sort_unstable();
    names.dedup();
    names
}

fn load_texture_catalog(ctx: &egui::Context) -> TextureCatalog {
    let Some(directory) = ensure_texture_directory() else {
        return TextureCatalog::default();
    };

    let names = scan_texture_names();
    let mut previews = Vec::new();

    for name in &names {
        let path = directory.join(format!("{}.png", name));

        let image = match image::open(&path) {
            Ok(image) => image,
            Err(error) => {
                eprintln!(
                    "[EDITOR][TEXTURES] Failed to load preview '{}': {}",
                    path.display(),
                    error
                );
                continue;
            }
        };

        let thumbnail = image.thumbnail(
            TEXTURE_PREVIEW_SIZE as u32,
            TEXTURE_PREVIEW_SIZE as u32,
        );
        let rgba = thumbnail.to_rgba8();

        let width = rgba.width() as usize;
        let height = rgba.height() as usize;

        if width == 0 || height == 0 {
            continue;
        }

        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [width, height],
            rgba.as_raw(),
        );

        let texture = ctx.load_texture(
            format!("aeoengine_texture_preview_{}", name),
            color_image,
            egui::TextureOptions::LINEAR,
        );

        previews.push(TexturePreview {
            name: name.clone(),
            texture,
        });
    }

    TextureCatalog { names, previews }
}

fn get_texture_catalog(ctx: &egui::Context) -> TextureCatalog {
    let id = egui::Id::new(TEXTURE_CATALOG_ID);

    if let Some(catalog) = ctx.data(|data| data.get_temp::<TextureCatalog>(id)) {
        return catalog;
    }

    let catalog = load_texture_catalog(ctx);

    ctx.data_mut(|data| {
        data.insert_temp(id, catalog.clone());
    });

    catalog
}

fn refresh_texture_catalog(ctx: &egui::Context) {
    let catalog = load_texture_catalog(ctx);

    ctx.data_mut(|data| {
        data.insert_temp(
            egui::Id::new(TEXTURE_CATALOG_ID),
            catalog,
        );
    });
}

fn open_texture_browser(ctx: &egui::Context) {
    refresh_texture_catalog(ctx);

    let id = egui::Id::new(TEXTURE_BROWSER_ID);

    ctx.data_mut(|data| {
        let mut state = data
            .get_temp::<TextureBrowserState>(id)
            .unwrap_or_default();

        state.open = true;

        data.insert_temp(id, state);
    });
}

// -----------------------------------------------------------------------------
// Shared asset search
// -----------------------------------------------------------------------------

fn asset_match_score(query: &str, name: &str) -> Option<u32> {
    let query = query.trim().to_lowercase();
    let name = name.to_lowercase();

    if query.is_empty() {
        return None;
    }

    if name == query {
        return Some(0);
    }

    if name.starts_with(&query) {
        return Some(10);
    }

    if name.contains(&query) {
        return Some(20);
    }

    let query_chars: Vec<char> = query.chars().collect();
    let name_chars: Vec<char> = name.chars().collect();

    let mut query_index = 0;

    for character in name_chars {
        if query_index < query_chars.len()
            && character == query_chars[query_index]
        {
            query_index += 1;
        }
    }

    (query_index == query_chars.len()).then_some(30)
}

fn ranked_asset_matches<I>(
    names: I,
    query: &str,
    limit: usize,
) -> Vec<String>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut matches: Vec<(u32, String)> = names
        .into_iter()
        .filter_map(|name| {
            let name = name.as_ref();

            asset_match_score(query, name)
                .map(|score| (score, name.to_string()))
        })
        .collect();

    matches.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cmp(&b.1))
    });

    matches
        .into_iter()
        .take(limit)
        .map(|(_, name)| name)
        .collect()
}

fn texture_matches(
    catalog: &TextureCatalog,
    texture: &str,
) -> Vec<String> {
    ranked_asset_matches(&catalog.names, texture, 6)
}

// -----------------------------------------------------------------------------
// Color editing
// -----------------------------------------------------------------------------

/// Draw the shared RGB editor and color-wheel toggle.
pub fn draw_color_edit(
    ui: &mut Ui,
    editor: &mut Editor,
    target: ColorTarget,
    color: &mut Vec3,
) {
    ui.horizontal(|ui| {
        let mut text = format!(
            "{:.3}, {:.3}, {:.3}",
            color.x,
            color.y,
            color.z
        );

        let response = ui.add(
            egui::TextEdit::singleline(&mut text)
                .desired_width(140.0)
                .hint_text("R, G, B"),
        );

        if response.changed() {
            let values: Vec<f32> = text
                .split(|character: char| {
                    character == ',' || character.is_whitespace()
                })
                .filter_map(|part| part.parse::<f32>().ok())
                .collect();

            if values.len() == 3 {
                *color = Vec3::new(
                    values[0],
                    values[1],
                    values[2],
                );
            }
        }

        response.context_menu(|ui| {
            if ui.button("Copy RGB Value").clicked() {
                ui.output_mut(|output| {
                    output.copied_text = format!(
                        "{:.3}, {:.3}, {:.3}",
                        color.x,
                        color.y,
                        color.z
                    );
                });

                ui.close_menu();
            }
        });

        ui.label("|");

        let is_open =
            editor.active_color_target == Some(target);

        let button = if is_open {
            egui::Button::new(
                RichText::new(" --- ").color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(0, 100, 200))
        } else {
            egui::Button::new(" --- ")
        };

        if ui.add(button).clicked() {
            editor.active_color_target = if is_open {
                None
            } else {
                Some(target)
            };
        }
    });
}

// -----------------------------------------------------------------------------
// Texture editing
// -----------------------------------------------------------------------------

/// Draw texture controls inside the Rendering menu.
pub fn draw_texture_edit(
    ui: &mut Ui,
    texture: &mut String,
) {
    let catalog = get_texture_catalog(ui.ctx());

    ui.horizontal(|ui| {
        ui.label("Texture:");

        ui.add(
            egui::TextEdit::singleline(texture)
                .desired_width(120.0)
                .hint_text("Texture name"),
        );

        if ui.button("Browse...").clicked() {
            open_texture_browser(ui.ctx());
        }

        if ui.button("Import...").clicked() {
            import_texture(ui.ctx(), texture);
        }
    });

    if texture.trim().is_empty() {
        return;
    }

    let matches = texture_matches(&catalog, texture);

    if matches.is_empty() {
        return;
    }

    ui.add_space(3.0);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(120.0);

        for match_name in matches {
            let selected = match_name == *texture;

            if ui
                .add(egui::SelectableLabel::new(
                    selected,
                    format!("{}.tx", match_name),
                ))
                .clicked()
            {
                *texture = match_name;
            }
        }
    });
}

/// Import a PNG texture and select it for the current build template.
fn import_texture(
    ctx: &egui::Context,
    texture: &mut String,
) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("PNG Image", &["png"])
        .pick_file()
    else {
        return;
    };

    let Some(textures_dir) = ensure_texture_directory() else {
        return;
    };

    let Some(file_name) = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
    else {
        eprintln!(
            "[EDITOR][TEXTURES] Imported file has no filename."
        );
        return;
    };

    let Some(stem) = path
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
    else {
        eprintln!(
            "[EDITOR][TEXTURES] Imported file has no valid name."
        );
        return;
    };

    let mut destination =
        textures_dir.join(&file_name);

    if destination.exists() {
        let mut index = 1;

        loop {
            let candidate =
                textures_dir.join(
                    format!("{}_{}.png", stem, index),
                );

            if !candidate.exists() {
                destination = candidate;
                break;
            }

            index += 1;
        }
    }

    match fs::copy(&path, &destination) {
        Ok(_) => {
            if let Some(final_stem) = destination
                .file_stem()
                .map(|name| name.to_string_lossy().to_string())
            {
                *texture = final_stem;
                refresh_texture_catalog(ctx);
            }
        }

        Err(error) => {
            eprintln!(
                "[EDITOR][TEXTURES] Failed to copy '{}' -> '{}': {}",
                path.display(),
                destination.display(),
                error
            );
        }
    }
}

// -----------------------------------------------------------------------------
// Texture browser
// -----------------------------------------------------------------------------

fn draw_texture_browser(
    ctx: &egui::Context,
    selected_texture: &mut String,
) {
    let id = egui::Id::new(TEXTURE_BROWSER_ID);

    let mut state = ctx
        .data(|data| {
            data.get_temp::<TextureBrowserState>(id)
        })
        .unwrap_or_default();

    if !state.open {
        return;
    }

    let mut open = state.open;
    let mut selected_texture_name = None;

    egui::Window::new("Texture Browser")
        .open(&mut open)
        .default_size(egui::vec2(700.0, 500.0))
        .min_size(egui::vec2(420.0, 320.0))
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Search:");

                ui.add(
                    egui::TextEdit::singleline(
                        &mut state.search,
                    )
                    .desired_width(240.0)
                    .hint_text("Filter textures..."),
                );

                if ui.button("Refresh").clicked() {
                    refresh_texture_catalog(ctx);
                }
            });

            ui.separator();

            let catalog =
                get_texture_catalog(ctx);

            let search =
                state.search.trim().to_lowercase();

            let previews: Vec<&TexturePreview> =
                catalog
                    .previews
                    .iter()
                    .filter(|preview| {
                        search.is_empty()
                            || preview
                                .name
                                .to_lowercase()
                                .contains(&search)
                    })
                    .collect();

            if previews.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);

                    if catalog.previews.is_empty() {
                        ui.label("No PNG textures found.");
                    } else {
                        ui.label(
                            "No textures match the search.",
                        );
                    }
                });

                return;
            }

            let available_width =
                ui.available_width()
                    .max(TEXTURE_TILE_WIDTH);

            let columns =
                ((available_width
                    + TEXTURE_TILE_SPACING)
                    / (TEXTURE_TILE_WIDTH
                        + TEXTURE_TILE_SPACING))
                .floor()
                .max(1.0) as usize;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for row in previews.chunks(columns) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing =
                                egui::vec2(
                                    TEXTURE_TILE_SPACING,
                                    8.0,
                                );

                            for preview in row {
                                ui.vertical(|ui| {
                                    ui.set_min_width(
                                        TEXTURE_TILE_WIDTH,
                                    );

                                    let [width, height] =
                                        preview.texture.size();

                                    let aspect_ratio =
                                        if height == 0 {
                                            1.0
                                        } else {
                                            width as f32 /
                                                height as f32
                                        };

                                    let mut image_width =
                                        TEXTURE_PREVIEW_SIZE;

                                    let mut image_height =
                                        TEXTURE_PREVIEW_SIZE;

                                    if aspect_ratio > 1.0 {
                                        image_height =
                                            image_width /
                                                aspect_ratio;
                                    } else {
                                        image_width =
                                            image_height *
                                                aspect_ratio;
                                    }

                                    let image_size =
                                        egui::vec2(
                                            image_width.max(1.0),
                                            image_height.max(1.0),
                                        );

                                    let image_button =
                                        egui::ImageButton::new((
                                            preview.texture.id(),
                                            image_size,
                                        ));

                                    if ui
                                        .add(image_button)
                                        .clicked()
                                    {
                                        selected_texture_name =
                                            Some(
                                                preview
                                                    .name
                                                    .clone(),
                                            );
                                    }

                                    ui.label(
                                        RichText::new(
                                            format!(
                                                "{}.tx",
                                                preview.name
                                            ),
                                        )
                                        .small(),
                                    );
                                });
                            }
                        });

                        ui.add_space(8.0);
                    }
                });
        });

    if let Some(name) = selected_texture_name {
        *selected_texture = name;
        open = false;
    }

    state.open = open;

    ctx.data_mut(|data| {
        data.insert_temp(id, state);
    });
}

// -----------------------------------------------------------------------------
// Audio browser
// -----------------------------------------------------------------------------

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

fn scan_directory_wavs(
    current_dir: &Path,
    root_dir: &Path,
    names: &mut Vec<String>,
) {
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
            scan_directory_wavs(
                &path,
                root_dir,
                names,
            );
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let is_wav = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| {
                extension.eq_ignore_ascii_case("wav")
            })
            .unwrap_or(false);

        if !is_wav {
            continue;
        }

        if let Ok(relative_path) =
            path.strip_prefix(root_dir)
        {
            names.push(
                relative_path
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}

/// Returns relative paths within the audio asset roots.
///
/// Project assets are scanned in addition to built-in assets. The project
/// audio root is checked first by the runtime, so duplicate relative paths
/// resolve to the project asset.
fn scan_audio_names(
    project_path: &Option<PathBuf>,
) -> Vec<String> {
    let mut names = Vec::new();

    let builtin_dir =
        PathBuf::from(".assets/audio");

    if builtin_dir.exists() {
        scan_directory_wavs(
            &builtin_dir,
            &builtin_dir,
            &mut names,
        );
    }

    if let Some(project) = project_path {
        let project_dir =
            project.join(".assets").join("audio");

        if project_dir.exists() {
            scan_directory_wavs(
                &project_dir,
                &project_dir,
                &mut names,
            );
        }
    }

    names.sort_unstable();
    names.dedup();
    names
}

fn load_audio_catalog(
    project_path: &Option<PathBuf>,
) -> AudioCatalog {
    AudioCatalog {
        names: scan_audio_names(project_path),
    }
}

fn get_audio_catalog(
    ctx: &egui::Context,
    project_path: &Option<PathBuf>,
) -> AudioCatalog {
    let id = egui::Id::new(AUDIO_CATALOG_ID);

    if let Some(catalog) =
        ctx.data(|data| data.get_temp::<AudioCatalog>(id))
    {
        return catalog;
    }

    let catalog =
        load_audio_catalog(project_path);

    ctx.data_mut(|data| {
        data.insert_temp(
            id,
            catalog.clone(),
        );
    });

    catalog
}

fn refresh_audio_catalog(
    ctx: &egui::Context,
    project_path: &Option<PathBuf>,
) {
    let catalog =
        load_audio_catalog(project_path);

    ctx.data_mut(|data| {
        data.insert_temp(
            egui::Id::new(AUDIO_CATALOG_ID),
            catalog,
        );
    });
}

fn open_audio_browser(
    ctx: &egui::Context,
    project_path: &Option<PathBuf>,
) {
    refresh_audio_catalog(
        ctx,
        project_path,
    );

    let id =
        egui::Id::new(AUDIO_BROWSER_ID);

    ctx.data_mut(|data| {
        let mut state = data
            .get_temp::<AudioBrowserState>(id)
            .unwrap_or_default();

        state.open = true;

        data.insert_temp(id, state);
    });
}

fn audio_matches(
    catalog: &AudioCatalog,
    audio: &str,
) -> Vec<String> {
    ranked_asset_matches(
        &catalog.names,
        audio,
        6,
    )
}

/// Draw the audio asset controls.
pub fn draw_audio_edit(
    ui: &mut Ui,
    audio: &mut String,
    project_path: &Option<PathBuf>,
) {
    let catalog =
        get_audio_catalog(
            ui.ctx(),
            project_path,
        );

    ui.horizontal(|ui| {
        ui.label("Audio:");

        ui.add(
            egui::TextEdit::singleline(audio)
                .desired_width(140.0)
                .hint_text("Audio asset path"),
        );

        if ui.button("Browse...").clicked() {
            open_audio_browser(
                ui.ctx(),
                project_path,
            );
        }

        if ui.button("Import...").clicked() {
            import_audio(
                ui.ctx(),
                audio,
                project_path,
            );
        }
    });

    if audio.trim().is_empty() {
        return;
    }

    let matches =
        audio_matches(&catalog, audio);

    if matches.is_empty() {
        return;
    }

    ui.add_space(3.0);

    egui::Frame::group(ui.style()).show(
        ui,
        |ui| {
            ui.set_min_width(140.0);

            for match_name in matches {
                let selected =
                    match_name == *audio;

                if ui
                    .add(
                        egui::SelectableLabel::new(
                            selected,
                            match_name.clone(),
                        ),
                    )
                    .clicked()
                {
                    *audio = match_name;
                }
            }
        },
    );
}

/// Import a WAV file into the active project's audio directory.
///
/// When no project is active, the file is imported into the built-in
/// `.assets/audio` directory.
pub fn import_audio(
    ctx: &egui::Context,
    audio: &mut String,
    project_path: &Option<PathBuf>,
) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("WAV Audio", &["wav"])
        .pick_file()
    else {
        return;
    };

    let destination_dir =
        project_path
            .as_ref()
            .map(|project| {
                project
                    .join(".assets")
                    .join("audio")
            })
            .unwrap_or_else(|| {
                PathBuf::from(".assets/audio")
            });

    if let Err(error) =
        fs::create_dir_all(&destination_dir)
    {
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
        eprintln!(
            "[EDITOR][AUDIO] Imported file has no filename."
        );
        return;
    };

    let stem = path
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "audio".to_string());

    let mut destination =
        destination_dir.join(&file_name);

    if destination.exists() {
        let mut index = 1;

        loop {
            let candidate =
                destination_dir.join(
                    format!("{}_{}.wav", stem, index),
                );

            if !candidate.exists() {
                destination = candidate;
                break;
            }

            index += 1;
        }
    }

    match fs::copy(&path, &destination) {
        Ok(_) => {
            if let Ok(relative_path) =
                destination.strip_prefix(&destination_dir)
            {
                *audio = relative_path
                    .to_string_lossy()
                    .replace('\\', "/");
            } else if let Some(name) =
                destination.file_name()
            {
                *audio =
                    name.to_string_lossy().to_string();
            }

            refresh_audio_catalog(
                ctx,
                project_path,
            );
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

fn draw_audio_browser(
    ctx: &egui::Context,
    selected_audio: &mut String,
    project_path: &Option<PathBuf>,
) {
    let id =
        egui::Id::new(AUDIO_BROWSER_ID);

    let mut state = ctx
        .data(|data| {
            data.get_temp::<AudioBrowserState>(id)
        })
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
                    egui::TextEdit::singleline(
                        &mut state.search,
                    )
                    .desired_width(240.0)
                    .hint_text("Filter audio assets..."),
                );

                if ui.button("Refresh").clicked() {
                    refresh_audio_catalog(
                        ctx,
                        project_path,
                    );
                }
            });

            ui.separator();

            let catalog =
                get_audio_catalog(
                    ctx,
                    project_path,
                );

            let search =
                state.search.trim().to_lowercase();

            let names: Vec<&String> =
                catalog
                    .names
                    .iter()
                    .filter(|name| {
                        search.is_empty()
                            || name
                                .to_lowercase()
                                .contains(&search)
                    })
                    .collect();

            if names.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);

                    if catalog.names.is_empty() {
                        ui.label(
                            "No WAV audio assets found.",
                        );
                    } else {
                        ui.label(
                            "No audio assets match the search.",
                        );
                    }
                });

                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for name in names {
                        let selected =
                            selected_audio.as_str()
                                == name.as_str();

                        ui.horizontal(|ui| {
                            ui.label("🎵");

                            if ui
                                .selectable_label(
                                    selected,
                                    name,
                                )
                                .clicked()
                            {
                                chosen_audio_name =
                                    Some(name.clone());
                            }
                        });
                    }
                });
        });

    if let Some(name) =
        chosen_audio_name
    {
        *selected_audio = name;
        open = false;
    }

    state.open = open;

    ctx.data_mut(|data| {
        data.insert_temp(id, state);
    });
}

// -----------------------------------------------------------------------------
// Main toolbar
// -----------------------------------------------------------------------------

pub fn draw_tool_bar(
    ui: &mut Ui,
    editor: &mut Editor,
    project_path: &Option<PathBuf>,
) {
    ui.horizontal(|ui| {
        let scripts_button =
            if editor.show_script_workspace {
                egui::Button::new(
                    RichText::new("Scripts")
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(
                    0, 100, 200,
                ))
            } else {
                egui::Button::new("Scripts")
            };

        if ui
            .add(scripts_button)
            .clicked()
        {
            editor.show_script_workspace =
                !editor.show_script_workspace;
        }

        ui.separator();

        if editor.show_script_workspace {
            ui.label(
                RichText::new(
                    "Script Workspace Active",
                )
                .italics(),
            );
            return;
        }

        ui.label(
            RichText::new("Mode").strong(),
        );

        if ui
            .selectable_label(
                editor.mode == EditorMode::Editor,
                "Editor",
            )
            .clicked()
        {
            editor.mode = EditorMode::Editor;
        }

        if ui
            .selectable_label(
                editor.mode == EditorMode::Play,
                "Play",
            )
            .clicked()
        {
            editor.mode = EditorMode::Play;
        }

        ui.separator();

        ui.label(
            RichText::new("Tool").strong(),
        );

        draw_tool_button(
            ui,
            &mut editor.current_tool,
            EditorTool::Navigate,
            "Navigate",
        );

        draw_tool_button(
            ui,
            &mut editor.current_tool,
            EditorTool::Select,
            "Select",
        );

        draw_tool_button(
            ui,
            &mut editor.current_tool,
            EditorTool::Build,
            "Build",
        );

        draw_tool_button(
            ui,
            &mut editor.current_tool,
            EditorTool::Erase,
            "Erase",
        );

        ui.separator();

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Picking:").strong(),
            );

            let text =
                if editor.plane_picking {
                    "Plane [ ON ]"
                } else {
                    "Plane [ OFF ]"
                };

            if ui.button(text).clicked() {
                editor.plane_picking =
                    !editor.plane_picking;
            }
        });

        if editor.current_tool != EditorTool::Build {
            return;
        }

        ui.separator();

        ui.menu_button(
            RichText::new(format!(
                "Block: {:?} ▼",
                editor.build_template.cell_type
            ))
            .strong(),
            |ui| {
                if ui
                    .selectable_label(
                        editor.build_template.cell_type
                            == crate::world::CellType::Block,
                        "Cube",
                    )
                    .clicked()
                {
                    editor.build_template =
                        crate::world::Cell::new_block();
                    ui.close_menu();
                }

                if ui
                    .selectable_label(
                        editor.build_template.cell_type
                            == crate::world::CellType::SpawnPoint,
                        "Spawn point",
                    )
                    .clicked()
                {
                    editor.build_template =
                        crate::world::Cell::new_spawn_point();
                    ui.close_menu();
                }

                if ui
                    .selectable_label(
                        editor.build_template.cell_type
                            == crate::world::CellType::Light,
                        "Light",
                    )
                    .clicked()
                {
                    editor.build_template =
                        crate::world::Cell::new_light();
                    ui.close_menu();
                }

                if ui
                    .selectable_label(
                        editor.build_template.cell_type
                            == crate::world::CellType::AudioEmitter,
                        "Audio Emitter",
                    )
                    .clicked()
                {
                    editor.build_template =
                        crate::world::Cell::new_audio_emitter();
                    ui.close_menu();
                }
            },
        );

        ui.separator();

        ui.menu_button(
            "Rendering ▼",
            |ui| {
                ui.horizontal(|ui| {
                    ui.label("Color:");

                    let mut color =
                        editor.build_template.color_rgb;

                    draw_color_edit(
                        ui,
                        editor,
                        ColorTarget::Build,
                        &mut color,
                    );

                    editor.build_template.color_rgb =
                        color;
                });

                ui.checkbox(
                    &mut editor.build_template.visible,
                    "Visible",
                );

                draw_texture_edit(
                    ui,
                    &mut editor.build_template.texture,
                );
            },
        );

        ui.separator();

        ui.menu_button(
            "Physics ▼",
            |ui| {
                ui.checkbox(
                    &mut editor.build_template.solid,
                    "Solid",
                );

                ui.checkbox(
                    &mut editor.build_template.anchored,
                    "Anchored",
                );
            },
        );
    });

    // Browsers are drawn after the toolbar so they do not depend on
    // the lifetime of the toolbar's menus.
    draw_texture_browser(
        ui.ctx(),
        &mut editor.build_template.texture,
    );

    draw_audio_browser(
        ui.ctx(),
        &mut editor.build_template.audio,
        project_path,
    );
}

fn draw_tool_button(
    ui: &mut Ui,
    current: &mut EditorTool,
    tool: EditorTool,
    text: &str,
) {
    let active = *current == tool;

    let button = if active {
        egui::Button::new(
            RichText::new(text)
                .color(Color32::WHITE),
        )
        .fill(Color32::from_rgb(0, 100, 200))
    } else {
        egui::Button::new(text)
    };

    if ui.add(button).clicked() {
        *current = tool;
    }
}