use super::asset_search::ranked_asset_matches;
use egui::{RichText, Ui};
use std::fs;
use std::path::PathBuf;

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
pub(crate) struct TextureCatalog {
    pub names: Vec<String>,
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

        let thumbnail = image.thumbnail(TEXTURE_PREVIEW_SIZE as u32, TEXTURE_PREVIEW_SIZE as u32);
        let rgba = thumbnail.to_rgba8();

        let width = rgba.width() as usize;
        let height = rgba.height() as usize;

        if width == 0 || height == 0 {
            continue;
        }

        let color_image = egui::ColorImage::from_rgba_unmultiplied([width, height], rgba.as_raw());

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
        data.insert_temp(egui::Id::new(TEXTURE_CATALOG_ID), catalog);
    });
}

fn open_texture_browser(ctx: &egui::Context) {
    refresh_texture_catalog(ctx);

    let id = egui::Id::new(TEXTURE_BROWSER_ID);

    ctx.data_mut(|data| {
        let mut state = data.get_temp::<TextureBrowserState>(id).unwrap_or_default();

        state.open = true;

        data.insert_temp(id, state);
    });
}

fn texture_matches(catalog: &TextureCatalog, texture: &str) -> Vec<String> {
    ranked_asset_matches(&catalog.names, texture, 6)
}

/// Draw texture controls inside the Rendering menu.
pub fn draw_texture_edit(ui: &mut Ui, texture: &mut String) {
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
fn import_texture(ctx: &egui::Context, texture: &mut String) {
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
        eprintln!("[EDITOR][TEXTURES] Imported file has no filename.");
        return;
    };

    let Some(stem) = path
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
    else {
        eprintln!("[EDITOR][TEXTURES] Imported file has no valid name.");
        return;
    };

    let mut destination = textures_dir.join(&file_name);

    if destination.exists() {
        let mut index = 1;

        loop {
            let candidate = textures_dir.join(format!("{}_{}.png", stem, index));

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

pub(crate) fn draw_texture_browser(ctx: &egui::Context, selected_texture: &mut String) {
    let id = egui::Id::new(TEXTURE_BROWSER_ID);

    let mut state = ctx
        .data(|data| data.get_temp::<TextureBrowserState>(id))
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
                    egui::TextEdit::singleline(&mut state.search)
                        .desired_width(240.0)
                        .hint_text("Filter textures..."),
                );

                if ui.button("Refresh").clicked() {
                    refresh_texture_catalog(ctx);
                }
            });

            ui.separator();

            let catalog = get_texture_catalog(ctx);

            let search = state.search.trim().to_lowercase();

            let previews: Vec<&TexturePreview> = catalog
                .previews
                .iter()
                .filter(|preview| {
                    search.is_empty() || preview.name.to_lowercase().contains(&search)
                })
                .collect();

            if previews.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);

                    if catalog.previews.is_empty() {
                        ui.label("No PNG textures found.");
                    } else {
                        ui.label("No textures match the search.");
                    }
                });

                return;
            }

            let available_width = ui.available_width().max(TEXTURE_TILE_WIDTH);

            let columns = ((available_width + TEXTURE_TILE_SPACING)
                / (TEXTURE_TILE_WIDTH + TEXTURE_TILE_SPACING))
                .floor()
                .max(1.0) as usize;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for row in previews.chunks(columns) {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(TEXTURE_TILE_SPACING, 8.0);

                            for preview in row {
                                ui.vertical(|ui| {
                                    ui.set_min_width(TEXTURE_TILE_WIDTH);

                                    let [width, height] = preview.texture.size();

                                    let aspect_ratio = if height == 0 {
                                        1.0
                                    } else {
                                        width as f32 / height as f32
                                    };

                                    let mut image_width = TEXTURE_PREVIEW_SIZE;

                                    let mut image_height = TEXTURE_PREVIEW_SIZE;

                                    if aspect_ratio > 1.0 {
                                        image_height = image_width / aspect_ratio;
                                    } else {
                                        image_width = image_height * aspect_ratio;
                                    }

                                    let image_size =
                                        egui::vec2(image_width.max(1.0), image_height.max(1.0));

                                    let image_button =
                                        egui::ImageButton::new((preview.texture.id(), image_size));

                                    if ui.add(image_button).clicked() {
                                        selected_texture_name = Some(preview.name.clone());
                                    }

                                    ui.label(RichText::new(format!("{}.tx", preview.name)).small());
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
