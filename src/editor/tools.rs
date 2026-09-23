use super::editor::{ColorTarget, Editor, EditorTool};
use crate::engine::EditorMode;
use egui::{Color32, RichText, Ui};
use glam::Vec3;
use std::fs;
use std::path::{Path, PathBuf};

// -----------------------------------------------------------------------------
// Texture browser state
// -----------------------------------------------------------------------------
//
// The texture browser is deliberately stored in egui temporary state instead
// of Editor so this change does not require modifying the Editor struct.
//
// The important part is that the browser is NOT drawn inside the Rendering
// menu. The Rendering menu only opens it. The browser itself is drawn after
// the toolbar finishes, so clicking inside the browser cannot close the menu.
// -----------------------------------------------------------------------------

const TEXTURE_BROWSER_ID: &str = "aeoengine_texture_browser";
const TEXTURE_CATALOG_ID: &str = "aeoengine_texture_catalog";

const TEXTURE_PREVIEW_SIZE: f32 = 88.0;
const TEXTURE_TILE_WIDTH: f32 = 112.0;
const TEXTURE_TILE_SPACING: f32 = 10.0;

/// A texture name plus its preview image.
#[derive(Clone)]
struct TexturePreview {
    name: String,
    texture: egui::TextureHandle,
}

/// Cached texture information.
///
/// This is intentionally kept out of the normal toolbar rendering path.
/// The filesystem is only scanned when the cache is missing or explicitly
/// refreshed.
#[derive(Clone, Default)]
struct TextureCatalog {
    names: Vec<String>,
    previews: Vec<TexturePreview>,
}

/// Persistent browser state.
#[derive(Clone, Default)]
struct TextureBrowserState {
    open: bool,
    search: String,
}

/// Return the location of the texture directory.
fn texture_directory() -> PathBuf {
    Path::new(".assets/textures").to_path_buf()
}

/// Make sure the texture directory exists.
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

/// Read available PNG texture names.
///
/// This does filesystem work only when explicitly requested by the texture
/// catalog loader.
fn scan_texture_names() -> Vec<String> {
    let Some(directory) = ensure_texture_directory() else {
        return Vec::new();
    };

    let mut names = Vec::new();

    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!(
                "[EDITOR][TEXTURES] Failed to read '{}': {}",
                directory.display(),
                error
            );

            return names;
        }
    };

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

/// Load the texture catalog.
///
/// Texture names and preview images are generated together so the browser
/// does not repeatedly hit the filesystem.
///
/// Preview images are reduced to a small size before being uploaded to egui.
/// A huge source texture therefore does not become a huge editor thumbnail.
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

/// Retrieve the cached texture catalog.
///
/// The catalog is loaded once on first use and then reused.
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

/// Force a texture catalog rebuild.
///
/// This is called after importing a texture or when the user presses Refresh.
fn refresh_texture_catalog(ctx: &egui::Context) {
    let catalog = load_texture_catalog(ctx);

    ctx.data_mut(|data| {
        data.insert_temp(
            egui::Id::new(TEXTURE_CATALOG_ID),
            catalog,
        );
    });
}

/// Open the custom texture browser.
///
/// The browser starts with a fresh catalog so newly added textures appear
/// immediately.
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

/// Score a texture name against the current text.
///
/// Lower scores are better.
///
/// Exact and prefix matches are preferred. Contains matches follow those.
/// A small fuzzy fallback allows obvious close names to appear without
/// modifying what the user typed.
fn texture_match_score(query: &str, name: &str) -> Option<u32> {
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

    // Lightweight fuzzy matching.
    //
    // We only use this as a fallback and never replace the user's input.
    let query_chars: Vec<char> = query.chars().collect();
    let name_chars: Vec<char> = name.chars().collect();

    if query_chars.is_empty() {
        return None;
    }

    let mut query_index = 0;

    for character in name_chars {
        if query_index < query_chars.len()
            && character == query_chars[query_index]
        {
            query_index += 1;
        }
    }

    if query_index == query_chars.len() {
        return Some(30);
    }

    None
}

/// Find the closest texture names for the inline suggestion list.
///
/// The result is deliberately capped so a large texture library cannot turn
/// the Rendering menu into a giant list.
fn texture_matches(
    catalog: &TextureCatalog,
    texture: &str,
) -> Vec<String> {
    let mut matches: Vec<(u32, String)> = catalog
        .names
        .iter()
        .filter_map(|name| {
            texture_match_score(texture, name)
                .map(|score| (score, name.clone()))
        })
        .collect();

    matches.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cmp(&b.1))
    });

    matches
        .into_iter()
        .take(6)
        .map(|(_, name)| name)
        .collect()
}

// -----------------------------------------------------------------------------
// Color editing
// -----------------------------------------------------------------------------

/// Unified color control with numeric entry and a toggle for a persistent
/// color wheel.
///
/// Format:
/// [ R, G, B | --- ]
pub fn draw_color_edit(
    ui: &mut Ui,
    editor: &mut Editor,
    target: ColorTarget,
    color: &mut Vec3,
) {
    ui.horizontal(|ui| {
        // --- Left side: Editable numeric values ---
        let mut text = format!(
            "{:.3}, {:.3}, {:.3}",
            color.x,
            color.y,
            color.z
        );

        let text_edit = egui::TextEdit::singleline(&mut text)
            .desired_width(140.0)
            .hint_text("R, G, B");

        let response = ui.add(text_edit);

        if response.changed() {
            let parts: Vec<f32> = text
                .split(|character: char| {
                    character == ',' || character.is_whitespace()
                })
                .filter_map(|part| part.parse::<f32>().ok())
                .collect();

            if parts.len() == 3 {
                *color = Vec3::new(
                    parts[0],
                    parts[1],
                    parts[2],
                );
            }
        }

        // Clipboard Copy
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

        // --- Right side: Color Wheel toggle button ---
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
            if is_open {
                editor.active_color_target = None;
            } else {
                editor.active_color_target = Some(target);
            }
        }
    });
}

// -----------------------------------------------------------------------------
// Texture editing
// -----------------------------------------------------------------------------

/// Draw the texture controls inside the existing Rendering menu.
///
/// The important design decision here is that the close-match list is NOT a
/// ComboBox or popup. It is simply part of the same Rendering menu.
///
/// That means clicking a suggestion cannot be mistaken for clicking outside
/// the Rendering menu.
pub fn draw_texture_edit(
    ui: &mut Ui,
    texture: &mut String,
) {
    let catalog = get_texture_catalog(ui.ctx());

    ui.horizontal(|ui| {
        ui.label("Texture:");

        // Direct texture name entry.
        //
        // The user's text is NEVER automatically replaced while typing.
        ui.add(
            egui::TextEdit::singleline(texture)
                .desired_width(120.0)
                .hint_text("Texture name"),
        );

        // Open the custom texture browser.
        if ui.button("Browse...").clicked() {
            open_texture_browser(ui.ctx());
        }

        // Existing import workflow.
        if ui.button("Import...").clicked() {
            import_texture(ui.ctx(), texture);
        }
    });

    // -------------------------------------------------------------------------
    // Close-match list
    // -------------------------------------------------------------------------
    //
    // This behaves like a dropdown, but it is NOT a second popup.
    //
    // Example:
    //
    // Texture: [ brick_0 ]
    //           brick_01
    //           brick_02
    //           brick_03
    //
    // Nothing is selected automatically.
    if !texture.trim().is_empty() {
        let matches = texture_matches(&catalog, texture);

        if !matches.is_empty() {
            ui.add_space(3.0);

            egui::Frame::group(ui.style())
                .show(ui, |ui| {
                    ui.set_min_width(120.0);

                    for match_name in matches {
                        let selected =
                            match_name == texture.as_str();

                        let label = egui::SelectableLabel::new(
                            selected,
                            format!("{}.tx", match_name),
                        );

                        if ui.add(label).clicked() {
                            // Only a click changes the field.
                            *texture = match_name;
                        }
                    }
                });
        }
    }
}

/// Import a PNG texture into the texture directory.
///
/// The imported texture is immediately selected by the build tool.
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

    let mut destination = textures_dir.join(&file_name);

    // Avoid overwriting an existing texture.
    if destination.exists() {
        let mut index = 1;

        loop {
            let candidate = textures_dir.join(
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
                // Automatically select the imported texture.
                *texture = final_stem;

                // Rebuild the cached names/previews now that the asset
                // directory has changed.
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
// Texture browser window
// -----------------------------------------------------------------------------

/// Draw the custom texture browser.
///
/// This function is called AFTER the main toolbar has finished rendering.
///
/// Therefore it is independent of the Rendering menu's popup lifetime.
fn draw_texture_browser(
    ctx: &egui::Context,
    selected_texture: &mut String,
) {
    let id = egui::Id::new(TEXTURE_BROWSER_ID);

    let mut state = ctx
        .data(|data| data.get_temp::<TextureBrowserState>(id))
        .unwrap_or_default();

    if !state.open {
        return;
    }

    let mut open = state.open;
    let mut selected_texture_name: Option<String> = None;

    egui::Window::new("Texture Browser")
        .open(&mut open)
        .default_size(egui::vec2(700.0, 500.0))
        .min_size(egui::vec2(420.0, 320.0))
        .resizable(true)
        .show(ctx, |ui| {
            // -----------------------------------------------------------------
            // Search / toolbar
            // -----------------------------------------------------------------

            ui.horizontal(|ui| {
                ui.label("Search:");

                ui.add(
                    egui::TextEdit::singleline(&mut state.search)
                        .desired_width(240.0)
                        .hint_text("Filter textures..."),
                );

                if ui.button("Refresh").clicked() {
                    refresh_texture_catalog(ctx);
                }
            });

            ui.separator();

            // Refreshing above may have replaced the catalog.
            let catalog = get_texture_catalog(ctx);

            let search = state.search.trim().to_lowercase();

            let mut previews: Vec<&TexturePreview> = catalog
                .previews
                .iter()
                .filter(|preview| {
                    search.is_empty()
                        || preview.name.to_lowercase().contains(&search)
                })
                .collect();

            previews.sort_by(|a, b| {
                a.name.cmp(&b.name)
            });

            if previews.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);

                    if catalog.previews.is_empty() {
                        ui.label("No PNG textures found.");
                    } else {
                        ui.label("No textures match the search.");
                    }
                });
            } else {
                // -----------------------------------------------------------------
                // Compact texture grid
                // -----------------------------------------------------------------
                //
                // The grid calculates its number of columns from the available
                // width instead of stretching each image across the whole window.
                //
                // This keeps thumbnails close together and avoids the huge gaps
                // from the previous implementation.
                let available_width =
                    ui.available_width().max(TEXTURE_TILE_WIDTH);

                let columns = ((available_width
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
                                // Keep the browser grid compact.
                                ui.spacing_mut().item_spacing =
                                    egui::vec2(
                                        TEXTURE_TILE_SPACING,
                                        8.0,
                                    );

                                for preview in row {
                                    ui.vertical(|ui| {
                                        ui.set_min_width(
                                            TEXTURE_TILE_WIDTH
                                        );

                                        let [width, height] =
                                            preview.texture.size();

                                        let aspect_ratio =
                                            if height == 0 {
                                                1.0
                                            } else {
                                                width as f32
                                                    / height as f32
                                            };

                                        let mut image_width =
                                            TEXTURE_PREVIEW_SIZE;

                                        let mut image_height =
                                            TEXTURE_PREVIEW_SIZE;

                                        if aspect_ratio > 1.0 {
                                            image_height =
                                                image_width
                                                    / aspect_ratio;
                                        } else {
                                            image_width =
                                                image_height
                                                    * aspect_ratio;
                                        }

                                        let image_size =
                                            egui::vec2(
                                                image_width.max(1.0),
                                                image_height.max(1.0),
                                            );

                                        // Clicking the picture selects the
                                        // texture for the Build tool.
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
                                                    preview.name.clone(),
                                                );
                                        }

                                        // Display the requested .tx name.
                                        ui.horizontal(|ui| {
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
                                    });
                                }
                            });

                            ui.add_space(8.0);
                        }
                    });
            }
        });

    // Selecting an image closes the browser and updates the build template.
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
// Audio browser state
// -----------------------------------------------------------------------------

const AUDIO_BROWSER_ID: &str = "aeoengine_audio_browser";
const AUDIO_CATALOG_ID: &str = "aeoengine_audio_catalog";

/// Cached audio asset names (relative paths within .assets/audio).
#[derive(Clone, Default)]
struct AudioCatalog {
    names: Vec<String>,
}

/// Persistent audio browser state.
#[derive(Clone, Default)]
struct AudioBrowserState {
    open: bool,
    search: String,
}

/// Recursively scan a directory for .wav files and return relative paths.
fn scan_directory_wavs(current_dir: &Path, root_dir: &Path, names: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_directory_wavs(&path, root_dir, names);
            } else if path.is_file() {
                let is_wav = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("wav"))
                    .unwrap_or(false);

                if is_wav {
                    if let Ok(rel_path) = path.strip_prefix(root_dir) {
                        let rel_str = rel_path.to_string_lossy().replace('\\', "/");
                        names.push(rel_str);
                    }
                }
            }
        }
    }
}

/// Read available WAV audio asset paths.
fn scan_audio_names(project_path: &Option<PathBuf>) -> Vec<String> {
    let mut names = Vec::new();

    let builtin_dir = Path::new(".assets/audio");
    if builtin_dir.exists() {
        scan_directory_wavs(builtin_dir, builtin_dir, &mut names);
    }

    if let Some(proj) = project_path {
        let proj_dir = proj.join(".assets").join("audio");
        if proj_dir.exists() {
            scan_directory_wavs(&proj_dir, &proj_dir, &mut names);
        }
    }

    names.sort_unstable();
    names.dedup();
    names
}

/// Load the audio catalog.
fn load_audio_catalog(project_path: &Option<PathBuf>) -> AudioCatalog {
    let names = scan_audio_names(project_path);
    AudioCatalog { names }
}

/// Retrieve the cached audio catalog.
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

/// Force an audio catalog rebuild.
fn refresh_audio_catalog(ctx: &egui::Context, project_path: &Option<PathBuf>) {
    let catalog = load_audio_catalog(project_path);
    ctx.data_mut(|data| {
        data.insert_temp(egui::Id::new(AUDIO_CATALOG_ID), catalog);
    });
}

/// Open the custom audio browser window.
fn open_audio_browser(ctx: &egui::Context, project_path: &Option<PathBuf>) {
    refresh_audio_catalog(ctx, project_path);

    let id = egui::Id::new(AUDIO_BROWSER_ID);

    ctx.data_mut(|data| {
        let mut state = data
            .get_temp::<AudioBrowserState>(id)
            .unwrap_or_default();

        state.open = true;

        data.insert_temp(id, state);
    });
}

/// Score an audio asset path against the current text.
fn audio_match_score(query: &str, name: &str) -> Option<u32> {
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

    if query_chars.is_empty() {
        return None;
    }

    let mut query_index = 0;

    for character in name_chars {
        if query_index < query_chars.len() && character == query_chars[query_index] {
            query_index += 1;
        }
    }

    if query_index == query_chars.len() {
        return Some(30);
    }

    None
}

/// Find the closest audio names for the inline suggestion list.
fn audio_matches(catalog: &AudioCatalog, audio: &str) -> Vec<String> {
    let mut matches: Vec<(u32, String)> = catalog
        .names
        .iter()
        .filter_map(|name| audio_match_score(audio, name).map(|score| (score, name.clone())))
        .collect();

    matches.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    matches.into_iter().take(6).map(|(_, name)| name).collect()
}

/// Draw the audio controls (TextEdit, Browse..., Import..., and inline suggestions).
pub fn draw_audio_edit(
    ui: &mut Ui,
    audio: &mut String,
    project_path: &Option<PathBuf>,
) {
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

    if !audio.trim().is_empty() {
        let matches = audio_matches(&catalog, audio);

        if !matches.is_empty() {
            ui.add_space(3.0);

            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_min_width(140.0);

                for match_name in matches {
                    let selected = match_name == audio.as_str();

                    let label = egui::SelectableLabel::new(selected, &match_name);

                    if ui.add(label).clicked() {
                        *audio = match_name;
                    }
                }
            });
        }
    }
}

/// Import a WAV file into the audio directory.
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

    let dest_dir = if let Some(proj) = project_path {
        proj.join(".assets").join("audio")
    } else {
        PathBuf::from(".assets/audio")
    };

    if let Err(error) = fs::create_dir_all(&dest_dir) {
        eprintln!(
            "[EDITOR][AUDIO] Failed to create '{}': {}",
            dest_dir.display(),
            error
        );
        return;
    }

    let Some(file_name) = path.file_name().map(|name| name.to_string_lossy().to_string()) else {
        eprintln!("[EDITOR][AUDIO] Imported file has no filename.");
        return;
    };

    let mut destination = dest_dir.join(&file_name);

    if destination.exists() {
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "audio".to_string());
        let mut index = 1;
        loop {
            let candidate = dest_dir.join(format!("{}_{}.wav", stem, index));
            if !candidate.exists() {
                destination = candidate;
                break;
            }
            index += 1;
        }
    }

    match fs::copy(&path, &destination) {
        Ok(_) => {
            if let Ok(rel_path) = destination.strip_prefix(&dest_dir) {
                let rel_str = rel_path.to_string_lossy().replace('\\', "/");
                *audio = rel_str;
            } else if let Some(name) = destination.file_name().map(|n| n.to_string_lossy().to_string()) {
                *audio = name;
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

/// Draw the custom audio browser window.
fn draw_audio_browser(
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
    let mut chosen_audio_name: Option<String> = None;

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

            let mut names: Vec<&String> = catalog
                .names
                .iter()
                .filter(|name| search.is_empty() || name.to_lowercase().contains(&search))
                .collect();

            names.sort();

            if names.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);

                    if catalog.names.is_empty() {
                        ui.label("No WAV audio assets found.");
                    } else {
                        ui.label("No audio assets match the search.");
                    }
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for name in names {
                            let is_selected = selected_audio.as_str() == name.as_str();

                            ui.horizontal(|ui| {
                                ui.label("🎵");
                                if ui.selectable_label(is_selected, name).clicked() {
                                    chosen_audio_name = Some(name.clone());
                                }
                            });
                        }
                    });
            }
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

// -----------------------------------------------------------------------------
// Main toolbar
// -----------------------------------------------------------------------------

pub fn draw_tool_bar(
    ui: &mut Ui,
    editor: &mut Editor,
    project_path: &Option<PathBuf>,
) {
    // Draw the actual toolbar first.
    //
    // The texture browser is intentionally drawn OUTSIDE this closure at the
    // end of the function. This is what prevents the Rendering menu from
    // controlling the browser's lifetime.
    ui.horizontal(|ui| {
        // --- Scripts button ---
        let scripts_btn = if editor.show_script_workspace {
            egui::Button::new(
                RichText::new("Scripts").color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(0, 100, 200))
        } else {
            egui::Button::new("Scripts")
        };

        if ui.add(scripts_btn).clicked() {
            editor.show_script_workspace =
                !editor.show_script_workspace;
        }

        ui.separator();

        // --- Script workspace ---
        if editor.show_script_workspace {
            ui.label(
                RichText::new("Script Workspace Active")
                    .italics(),
            );

            return;
        }

        // --- Mode ---
        ui.label(RichText::new("Mode").strong());

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

        // --- Tool ---
        ui.label(RichText::new("Tool").strong());

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

        // --- Plane picking ---
        ui.horizontal(|ui| {
            ui.label(RichText::new("Picking:").strong());

            let pick_text = if editor.plane_picking {
                "Plane [ ON ]"
            } else {
                "Plane [ OFF ]"
            };

            if ui.button(pick_text).clicked() {
                editor.plane_picking =
                    !editor.plane_picking;
            }
        });

        // --- Build tool settings ---
        if editor.current_tool == EditorTool::Build {
            ui.separator();

            // Block Type Dropdown
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

            // Rendering Settings Dropdown
            //
            // This remains a normal menu_button so the top toolbar keeps
            // exactly the same visual behavior as the other working buttons.
            ui.menu_button("Rendering ▼", |ui| {
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

                    editor.build_template.color_rgb = color;
                });

                ui.checkbox(
                    &mut editor.build_template.visible,
                    "Visible",
                );

                draw_texture_edit(
                    ui,
                    &mut editor.build_template.texture,
                );
            });

            ui.separator();

            // Physics Settings Dropdown
            ui.menu_button("Physics ▼", |ui| {
                ui.checkbox(
                    &mut editor.build_template.solid,
                    "Solid",
                );

                ui.checkbox(
                    &mut editor.build_template.anchored,
                    "Anchored",
                );
            });
        }
    });

    // -------------------------------------------------------------------------
    // Texture & Audio browsers
    // -------------------------------------------------------------------------
    //
    // IMPORTANT:
    // This is intentionally outside the toolbar's ui.horizontal closure.
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

/// Draw one of the main editor tool buttons.
fn draw_tool_button(
    ui: &mut Ui,
    current: &mut EditorTool,
    tool: EditorTool,
    text: &str,
) {
    let is_active = *current == tool;

    let button = if is_active {
        egui::Button::new(
            RichText::new(text).color(Color32::WHITE),
        )
        .fill(Color32::from_rgb(0, 100, 200))
    } else {
        egui::Button::new(text)
    };

    if ui.add(button).clicked() {
        *current = tool;
    }
}
