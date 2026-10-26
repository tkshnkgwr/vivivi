//! モーダルウィンドウ（ヘルプ・印刷/Diffプレビュー・クリップボード履歴・最近使ったファイル・2ペインファイラー）描画モジュール。

use eframe::egui;
use std::path::PathBuf;

use super::ViviviGuiApp;
use crate::theme::colors;

impl ViviviGuiApp {
    /// 共通 `App` コアのモーダル状態（ヘルプ・印刷プレビュー・履歴・クリップボード・ファイラー）を
    /// フローティングウィンドウとして描画します。
    pub(crate) fn render_modals(&mut self, ctx: &egui::Context) {
        // 5.1 ヘルプビューア (:help / F1)
        if self.core.show_help {
            let mut open = self.core.show_help;
            let title = if self.core.help_title.is_empty() {
                "vivivi Help".to_string()
            } else {
                self.core.help_title.clone()
            };
            egui::Window::new(title)
                .open(&mut open)
                .default_size([620.0, 460.0])
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for line in self.core.help_lines.iter().skip(self.core.help_scroll) {
                            ui.label(
                                egui::RichText::new(line)
                                    .color(colors::FG_TEXT)
                                    .family(egui::FontFamily::Monospace),
                            );
                        }
                    });
                });
            self.core.show_help = open;
        }

        // 5.2 印刷・出力プレビュー (:preview / :todo / :diff 等)
        if self.core.show_preview {
            let mut open = self.core.show_preview;
            egui::Window::new("Preview / Output")
                .open(&mut open)
                .default_size([640.0, 480.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Export PDF (p)").clicked()
                            && let Err(e) = self.core.export_pdf(None)
                        {
                            self.core.status_message = Some(format!("PDF出力エラー: {e}"));
                        }
                    });
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for line in self
                            .core
                            .preview_lines
                            .iter()
                            .skip(self.core.preview_scroll)
                        {
                            ui.label(
                                egui::RichText::new(line)
                                    .color(colors::FG_TEXT)
                                    .family(egui::FontFamily::Monospace),
                            );
                        }
                    });
                });
            self.core.show_preview = open;
        }

        // 5.3 クリップボード履歴 (F6)
        if self.core.show_clipboard_hist {
            let mut open = self.core.show_clipboard_hist;
            egui::Window::new("Clipboard History (F6)")
                .open(&mut open)
                .default_size([480.0, 320.0])
                .show(ctx, |ui| {
                    let items: Vec<String> = self
                        .core
                        .vi
                        .registers()
                        .history()
                        .items()
                        .iter()
                        .map(|rc| rc.content.preview_text(60))
                        .collect();
                    if items.is_empty() {
                        ui.label(
                            egui::RichText::new("(履歴はありません)").color(colors::FG_DIMMED),
                        );
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (idx, preview) in items.iter().enumerate() {
                                let is_sel = idx == self.core.selected_cb_idx;
                                if ui
                                    .selectable_label(is_sel, format!("{}: {}", idx + 1, preview))
                                    .clicked()
                                {
                                    self.core.selected_cb_idx = idx;
                                }
                            }
                        });
                    }
                });
            self.core.show_clipboard_hist = open;
        }

        // 5.4 最近使ったファイル ＆ お気に入り (:recent / F7)
        if self.core.show_recent_modal {
            let mut open = self.core.show_recent_modal;
            let mut file_to_open: Option<PathBuf> = None;
            egui::Window::new("Recent & Favorite Files (F7)")
                .open(&mut open)
                .default_size([520.0, 360.0])
                .show(ctx, |ui| {
                    let mut all_entries = Vec::new();
                    for fav in &self.core.recent_manager.favorite_files {
                        all_entries.push((true, fav.clone()));
                    }
                    for rec in &self.core.recent_manager.recent_files {
                        if !self.core.recent_manager.favorite_files.contains(rec) {
                            all_entries.push((false, rec.clone()));
                        }
                    }

                    if all_entries.is_empty() {
                        ui.label(
                            egui::RichText::new("(履歴はありません)").color(colors::FG_DIMMED),
                        );
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (idx, (is_fav, path)) in all_entries.iter().enumerate() {
                                let icon = if *is_fav { "★" } else { "📄" };
                                let label = format!("{} {}", icon, path.display());
                                if ui
                                    .selectable_label(idx == self.core.selected_recent_idx, label)
                                    .clicked()
                                {
                                    file_to_open = Some(path.clone());
                                }
                            }
                        });
                    }
                });
            if let Some(target) = file_to_open {
                open = false;
                self.core
                    .execute_ex(&format!("e {}", target.to_string_lossy()));
            }
            self.core.show_recent_modal = open;
        }

        // 5.5 2ペイン・ファイラー (:filer)
        if self.core.show_filer {
            let mut open = self.core.show_filer;
            egui::Window::new("2-Pane Filer (:filer)")
                .open(&mut open)
                .default_size([680.0, 440.0])
                .show(ctx, |ui| {
                    if let Some(ref filer) = self.core.filer {
                        ui.columns(2, |cols| {
                            cols[0].label(
                                egui::RichText::new(format!(
                                    "Left: {}",
                                    filer.left.current_dir.display()
                                ))
                                .strong(),
                            );
                            cols[0].separator();
                            egui::ScrollArea::vertical().id_salt("filer_left").show(
                                &mut cols[0],
                                |ui| {
                                    for (i, entry) in filer.left.entries.iter().enumerate() {
                                        let prefix = if entry.is_dir { "📁" } else { "📄" };
                                        let is_left_active = filer.active
                                            == vivivi_modes::filer::ActiveFilerPane::Left;
                                        let _ = ui.selectable_label(
                                            is_left_active && i == filer.left.cursor,
                                            format!("{} {}", prefix, entry.name),
                                        );
                                    }
                                },
                            );

                            cols[1].label(
                                egui::RichText::new(format!(
                                    "Right: {}",
                                    filer.right.current_dir.display()
                                ))
                                .strong(),
                            );
                            cols[1].separator();
                            egui::ScrollArea::vertical().id_salt("filer_right").show(
                                &mut cols[1],
                                |ui| {
                                    for (i, entry) in filer.right.entries.iter().enumerate() {
                                        let prefix = if entry.is_dir { "📁" } else { "📄" };
                                        let is_right_active = filer.active
                                            == vivivi_modes::filer::ActiveFilerPane::Right;
                                        let _ = ui.selectable_label(
                                            is_right_active && i == filer.right.cursor,
                                            format!("{} {}", prefix, entry.name),
                                        );
                                    }
                                },
                            );
                        });
                    }
                });
            if !open {
                self.core.close_filer();
            }
        }
    }
}
