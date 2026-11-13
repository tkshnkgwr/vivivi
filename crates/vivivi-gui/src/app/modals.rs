//! モーダルウィンドウ（ヘルプ・印刷/Diffプレビュー・クリップボード履歴・最近使ったファイル・2ペインファイラー）描画モジュール。

use eframe::egui;
use std::path::PathBuf;

use super::ViviviGuiApp;

impl ViviviGuiApp {
    /// いずれかのモーダルウィンドウが開いているかどうかを返します。
    pub fn is_any_modal_open(&self) -> bool {
        self.core.show_help
            || self.core.show_preview
            || self.core.show_clipboard_hist
            || self.core.show_recent_modal
            || self.core.show_filer
            || self.core.show_cmd_exec_dialog
            || self.settings_dialog.is_open
    }

    /// 開いているすべてのモーダルウィンドウを閉じます。
    pub fn close_all_modals(&mut self) {
        self.core.show_help = false;
        self.core.show_preview = false;
        self.core.show_clipboard_hist = false;
        self.core.show_recent_modal = false;
        self.core.lsp_hover_text = None;
        if self.core.show_filer {
            self.core.close_filer();
        }
        if self.core.show_cmd_exec_dialog {
            self.core.close_cmd_exec_dialog();
        }
        self.settings_dialog.close();
    }

    /// 共通 `App` コアのモーダル状態（ヘルプ・印刷プレビュー・履歴・クリップボード・ファイラー）を
    /// フローティングウィンドウとして描画します。
    ///
    /// # 引数
    /// - `ctx`: 描画先の `egui::Context`
    pub fn render_modals(&mut self, ctx: &egui::Context) {
        let pal = self.palette();
        // 5.1 ヘルプビューア (:help / F1)
        if self.core.show_help {
            let mut open = self.core.show_help;
            let title = if self.core.help_title.is_empty() {
                "vivivi Help".to_string()
            } else {
                self.core.help_title.clone()
            };
            let screen_rect = ctx.screen_rect();
            let help_width = (screen_rect.width() - 32.0).clamp(620.0, 1240.0);
            let help_height = (screen_rect.height() - 48.0).clamp(360.0, 560.0);
            egui::Window::new(title)
                .id(egui::Id::new("vivivi_help_modal_v2"))
                .open(&mut open)
                .default_size([help_width, help_height])
                .show(ctx, |ui| {
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for line in &self.core.help_lines {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(line)
                                            .color(pal.fg_text)
                                            .family(egui::FontFamily::Monospace),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
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
                        for line in &self.core.preview_lines {
                            ui.label(
                                egui::RichText::new(line)
                                    .color(pal.fg_text)
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
                        ui.label(egui::RichText::new("(履歴はありません)").color(pal.fg_dimmed));
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
                        ui.label(egui::RichText::new("(履歴はありません)").color(pal.fg_dimmed));
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
                if let Err(e) = self.core.open_file(target) {
                    self.core.status_message = Some(format!("ファイル読込エラー: {e}"));
                }
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

        // 5.6 LSP ホバーポップアップ (K / :lsp hover)
        if let Some(hover_text) = self.core.lsp_hover_text.clone() {
            let mut open = true;
            egui::Window::new("LSP Hover (K)")
                .open(&mut open)
                .default_size([480.0, 220.0])
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for line in hover_text.lines() {
                            ui.label(
                                egui::RichText::new(line)
                                    .color(pal.fg_text)
                                    .family(egui::FontFamily::Monospace),
                            );
                        }
                    });
                });
            if !open {
                self.core.lsp_hover_text = None;
            }
        }

        // 5.7 LSP 診断情報パネル (:lsp diag)
        if self.core.lsp_show_diagnostics_panel {
            let mut open = self.core.lsp_show_diagnostics_panel;
            egui::Window::new("LSP Diagnostics (:lsp diag)")
                .open(&mut open)
                .default_size([560.0, 240.0])
                .show(ctx, |ui| {
                    let mut jump_to: Option<(usize, usize)> = None;
                    {
                        let diags = if let (Some(mgr), Some(path)) =
                            (&self.core.lsp_manager, &self.core.file_path)
                        {
                            let uri =
                                format!("file:///{}", path.to_string_lossy().replace('\\', "/"));
                            mgr.get_diagnostics(&uri).unwrap_or(&[])
                        } else {
                            &[]
                        };

                        if diags.is_empty() {
                            ui.label(
                                egui::RichText::new("(診断エラー・警告はありません)")
                                    .color(pal.fg_dimmed),
                            );
                        } else {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for diag in diags {
                                    let (prefix, col) = match diag.severity {
                                        Some(vivivi_modes::lsp::DiagnosticSeverity::Error) => {
                                            ("[ERR]", pal.accent_red)
                                        }
                                        Some(vivivi_modes::lsp::DiagnosticSeverity::Warning) => {
                                            ("[WRN]", pal.accent_yellow)
                                        }
                                        _ => ("[INF]", pal.accent_cyan),
                                    };
                                    let line_no = diag.range.start.line as usize;
                                    let char_no = diag.range.start.character as usize;
                                    let label = format!(
                                        "{} Ln {}, Col {}: {}",
                                        prefix,
                                        line_no + 1,
                                        char_no + 1,
                                        diag.message
                                    );
                                    if ui
                                        .add(
                                            egui::Label::new(
                                                egui::RichText::new(label)
                                                    .color(col)
                                                    .family(egui::FontFamily::Monospace),
                                            )
                                            .sense(egui::Sense::click()),
                                        )
                                        .on_hover_text("クリックして該当行へジャンプ")
                                        .clicked()
                                    {
                                        jump_to = Some((line_no, char_no));
                                    }
                                }
                            });
                        }
                    }
                    if let Some((l, c)) = jump_to {
                        self.core.cursor.pos.line = l;
                        self.core.cursor.pos.col = c;
                        self.core.clamp_cursor();
                    }
                });
            self.core.lsp_show_diagnostics_panel = open;
        }

        // 5.8 外部コマンド実行ダイアログ (:cmd / :cmdexec)
        if self.core.show_cmd_exec_dialog {
            let mut open = self.core.show_cmd_exec_dialog;
            let mut should_execute = false;
            let mut should_close = false;
            egui::Window::new("外部コマンド実行 (cmdExecDlg)")
                .id(egui::Id::new("vivivi_cmd_exec_dialog"))
                .open(&mut open)
                .default_size([620.0, 420.0])
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new("実行モード")
                            .color(pal.fg_dimmed)
                            .strong(),
                    );
                    ui.horizontal_wrapped(|ui| {
                        let modes = [
                            (0usize, "1. 実行のみ (結果確認)"),
                            (1usize, "2. バッファ挿入 (:r!)"),
                            (2usize, "3. 選択行置換 (:[range]!)"),
                        ];
                        for (idx, label) in modes {
                            ui.selectable_value(&mut self.core.cmd_exec_mode, idx, label);
                        }
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("コマンド:").color(pal.fg_text));
                        let edit_resp = ui.add_sized(
                            [ui.available_width() - 140.0, 24.0],
                            egui::TextEdit::singleline(&mut self.core.cmd_exec_input)
                                .font(egui::FontId::monospace(13.0))
                                .hint_text("例: git status, date, sort"),
                        );
                        if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            should_execute = true;
                        }
                        if ui.button("▶ 実行 (Enter)").clicked() {
                            should_execute = true;
                        }
                        if ui.button("閉じる").clicked() {
                            should_close = true;
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.label(
                        egui::RichText::new("実行結果 / プレビュー")
                            .color(pal.fg_dimmed)
                            .size(12.0),
                    );
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let preview = self.core.cmd_exec_output_preview.as_deref().unwrap_or(
                                "(まだ実行されていません - コマンドを入力して Enter または「実行」を押してください)",
                            );
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(preview)
                                        .color(pal.fg_text)
                                        .family(egui::FontFamily::Monospace),
                                )
                                .wrap_mode(egui::TextWrapMode::Extend),
                            );
                        });
                });

            if should_execute {
                self.core.execute_cmd_exec_dialog();
            }
            if should_close || !open {
                self.core.close_cmd_exec_dialog();
            }
        }
    }
}
