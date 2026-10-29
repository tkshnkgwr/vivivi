//! メニューバー・マルチタブバー・左サイドバー・下部ステータスバー＆コマンドライン描画モジュール。

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, Margin, Sense, Vec2};
use std::path::PathBuf;
use vivivi_vi::ViMode;

use super::{SidebarTab, ViviviGuiApp};
use crate::theme::colors;

impl ViviviGuiApp {
    /// 1. メニューバー ＆ マルチタブバーの描画
    pub(crate) fn render_top_panel(&mut self, ctx: &egui::Context) {
        self.core.sync_active_tab_state();

        egui::TopBottomPanel::top("top_panel")
            .frame(Frame::NONE.fill(colors::BG_PANEL))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                    ui.visuals_mut().widgets.hovered.bg_fill = colors::BG_HOVER;

                    ui.menu_button("File", |ui| {
                        if ui.button("New Tab  (:tabnew)").clicked() {
                            self.core.new_tab(None);
                            ui.close_menu();
                        }
                        if ui.button("Save  (:w)").clicked() {
                            self.core.execute_ex("w");
                            ui.close_menu();
                        }
                        if ui.button("Recent Files  (:recent)").clicked() {
                            self.core.execute_ex("recent");
                            ui.close_menu();
                        }
                        if ui.button("2-Pane Filer  (:filer)").clicked() {
                            self.core.execute_ex("filer");
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Close Tab  (:tabclose)").clicked() {
                            let _ = self.core.close_tab();
                            ui.close_menu();
                        }
                        if ui.button("Quit  (:q)").clicked() {
                            self.core.execute_ex("q");
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Edit", |ui| {
                        if ui.button("Undo  (u)").clicked() {
                            self.core.execute_ex("undo");
                            ui.close_menu();
                        }
                        if ui.button("Redo  (Ctrl-R)").clicked() {
                            self.core.execute_ex("redo");
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Toggle Keisen Mode  (F5)").clicked() {
                            self.core.keisen_mode.toggle();
                            ui.close_menu();
                        }
                        if ui.button("Toggle CSV Mode  (:csv)").clicked() {
                            self.core.execute_ex("csv");
                            ui.close_menu();
                        }
                        if ui.button("Clipboard History  (F6)").clicked() {
                            self.core.show_clipboard_hist = !self.core.show_clipboard_hist;
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("View", |ui| {
                        if ui.checkbox(&mut self.sidebar_open, "Sidebar").clicked() {
                            ui.close_menu();
                        }
                        if ui.checkbox(&mut self.minimap_open, "Minimap").clicked() {
                            ui.close_menu();
                        }
                        if ui
                            .checkbox(&mut self.core.show_line_numbers, "Line Numbers")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .checkbox(&mut self.core.relative_number, "Relative Line Numbers")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .checkbox(
                                &mut self.core.list_chars_config.enabled,
                                "Invisible Chars (:set list)",
                            )
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .checkbox(&mut self.particles.enabled, "Delete Particles")
                            .clicked()
                        {
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Window", |ui| {
                        if ui.button("Split Horizontal  (:sp)").clicked() {
                            self.core.execute_ex("sp");
                            ui.close_menu();
                        }
                        if ui.button("Split Vertical  (:vsp)").clicked() {
                            self.core.execute_ex("vsp");
                            ui.close_menu();
                        }
                        if ui.button("Close Window  (:close)").clicked() {
                            self.core.execute_ex("close");
                            ui.close_menu();
                        }
                        if ui.button("Only Window  (:only)").clicked() {
                            self.core.execute_ex("only");
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Help", |ui| {
                        if ui.button("Vi Cheatsheet  (:help)").clicked() {
                            self.core.execute_ex("help");
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.label("vivivi - Vi-like Editor in Rust");
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let minimap_label = if self.minimap_open {
                            "🗺 Minimap"
                        } else {
                            "🗺 (Off)"
                        };
                        if ui
                            .selectable_label(self.minimap_open, minimap_label)
                            .clicked()
                        {
                            self.minimap_open = !self.minimap_open;
                        }

                        let sidebar_label = if self.sidebar_open {
                            "📁 Sidebar"
                        } else {
                            "📁 (Off)"
                        };
                        if ui
                            .selectable_label(self.sidebar_open, sidebar_label)
                            .clicked()
                        {
                            self.sidebar_open = !self.sidebar_open;
                        }
                    });
                });

                ui.separator();

                // 下段: マルチタブバー
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

                    let active_idx = self.core.active_tab;
                    let tab_count = self.core.tabs.len();
                    let mut tab_to_close = None;
                    let mut tab_to_switch = None;

                    for (idx, tab) in self.core.tabs.iter().enumerate() {
                        let is_active = idx == active_idx;

                        let file_name = tab
                            .file_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("[No Name]");

                        let dirty_badge = if tab.buffer.is_modified() { " •" } else { "" };
                        let tab_label = format!("{}{}", file_name, dirty_badge);

                        let bg_color = if is_active {
                            colors::BG_EDITOR
                        } else {
                            colors::BG_PANEL
                        };

                        let text_color = if is_active {
                            colors::FG_TEXT
                        } else {
                            colors::FG_DIMMED
                        };

                        Frame::NONE
                            .fill(bg_color)
                            .corner_radius(CornerRadius {
                                nw: 4,
                                ne: 4,
                                sw: 0,
                                se: 0,
                            })
                            .inner_margin(Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let btn = ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(tab_label)
                                                .color(text_color)
                                                .strong(),
                                        )
                                        .sense(Sense::click()),
                                    );

                                    if btn.clicked() {
                                        tab_to_switch = Some(idx);
                                    }

                                    if tab_count > 1 {
                                        let close_btn = ui.add(
                                            egui::Label::new(
                                                egui::RichText::new("×")
                                                    .color(colors::FG_DIMMED)
                                                    .size(12.0),
                                            )
                                            .sense(Sense::click()),
                                        );
                                        if close_btn.clicked() {
                                            tab_to_close = Some(idx);
                                        }
                                    }
                                });
                            });
                    }

                    let add_tab_btn = ui.add(
                        egui::Button::new(egui::RichText::new(" + ").color(colors::FG_DIMMED))
                            .frame(false),
                    );
                    if add_tab_btn.clicked() {
                        self.core.new_tab(None);
                    }

                    if let Some(idx) = tab_to_switch {
                        self.core.switch_tab_1based(idx + 1);
                    }
                    if let Some(idx) = tab_to_close {
                        if idx == self.core.active_tab {
                            let _ = self.core.close_tab();
                        } else if tab_count > 1 && idx < self.core.tabs.len() {
                            if self.core.tabs[idx].buffer.is_modified() {
                                self.core.status_message = Some(format!(
                                    "E37: 未保存の変更があります (タブ {})。先に保存してください",
                                    idx + 1
                                ));
                            } else {
                                self.core.tabs.remove(idx);
                                if self.core.active_tab > idx {
                                    self.core.active_tab -= 1;
                                }
                                self.core.sync_active_tab_state();
                            }
                        }
                    }
                });
            });
    }

    /// 2. ファイルツリー / アウトライン用折りたたみサイドバーの描画
    pub(crate) fn render_sidebar(&mut self, ctx: &egui::Context) {
        if !self.sidebar_open {
            return;
        }

        egui::SidePanel::left("sidebar_panel")
            .frame(
                Frame::NONE
                    .fill(colors::BG_PANEL)
                    .inner_margin(Margin::same(6)),
            )
            .default_width(self.sidebar_width)
            .width_range(160.0..=450.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.sidebar_tab, SidebarTab::Files, " 📁 Files ");
                    ui.selectable_value(&mut self.sidebar_tab, SidebarTab::Outline, " 🔖 Outline ");
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| match self.sidebar_tab {
                    SidebarTab::Files => {
                        ui.label(
                            egui::RichText::new("PROJECT EXPLORER")
                                .color(colors::FG_DIMMED)
                                .size(11.0),
                        );
                        ui.add_space(4.0);

                        let current_dir_name = self
                            .current_dir
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("workspace");

                        ui.label(
                            egui::RichText::new(format!("📂 {}", current_dir_name))
                                .strong()
                                .color(colors::FG_TEXT),
                        );

                        let mut dir_to_navigate: Option<PathBuf> = None;
                        let mut file_to_open: Option<PathBuf> = None;

                        // 親ディレクトリ (..) への移動リンク
                        if let Some(parent) = self.current_dir.parent() {
                            let parent_path = parent.to_path_buf();
                            let up_label = ui.add(
                                egui::Label::new(
                                    egui::RichText::new("  ⬆ ..").color(colors::ACCENT_BLUE),
                                )
                                .sense(Sense::click()),
                            );
                            if up_label.clicked() {
                                dir_to_navigate = Some(parent_path);
                            }
                        }

                        for path in &self.cached_files {
                            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            let is_dir = path.is_dir();
                            let icon = if is_dir { "📁" } else { "📄" };
                            let label_text = format!("  {} {}", icon, name);

                            let label = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(label_text).color(colors::FG_TEXT),
                                )
                                .sense(Sense::click()),
                            );

                            if label.double_clicked() || (is_dir && label.clicked()) {
                                if is_dir {
                                    dir_to_navigate = Some(path.clone());
                                } else {
                                    file_to_open = Some(path.clone());
                                }
                            }
                        }

                        if let Some(target_dir) = dir_to_navigate {
                            self.navigate_sidebar_dir(target_dir);
                        }
                        if let Some(target_path) = file_to_open
                            && let Err(e) = self.core.open_file(target_path)
                        {
                            self.core.status_message = Some(format!("ファイル読込エラー: {e}"));
                        }
                    }
                    SidebarTab::Outline => {
                        ui.label(
                            egui::RichText::new("SYMBOLS / OUTLINE")
                                .color(colors::FG_DIMMED)
                                .size(11.0),
                        );
                        ui.add_space(4.0);

                        if self.core.outline_items.is_empty() {
                            ui.label(
                                egui::RichText::new("(No symbols detected)")
                                    .color(colors::FG_DIMMED),
                            );
                        } else {
                            let mut jump_line = None;
                            for item in &self.core.outline_items {
                                let indent = "  ".repeat(item.level.saturating_sub(1));
                                let symbol_label =
                                    format!("{}L{}: {}", indent, item.line_idx + 1, item.title);
                                let label = ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(symbol_label).color(colors::FG_TEXT),
                                    )
                                    .sense(Sense::click()),
                                );
                                if label.clicked() {
                                    jump_line = Some(item.line_idx);
                                }
                            }
                            if let Some(line) = jump_line {
                                self.core.cursor.pos.line = line;
                                self.core.cursor.pos.col = 0;
                                self.core.clamp_cursor();
                                // 画面中央付近にジャンプ行が配置されるようスクロールを同期
                                self.core.scroll.0 = line.saturating_sub(12);
                            }
                        }
                    }
                });
            });
    }

    /// 3. ステータスバー ＆ 最下段コマンドラインの描画（`TextEdit` 不使用）
    pub(crate) fn render_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_status_panel")
            .frame(Frame::NONE.fill(colors::BG_PANEL))
            .show(ctx, |ui| {
                // 上段: ステータスライン
                Frame::NONE
                    .fill(colors::BG_STATUSLINE)
                    .inner_margin(Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mode = self.core.mode();
                            let (badge_bg, badge_fg) = self.mode_badge_color(mode);
                            let mode_name = format!(" {} ", mode.name());

                            Frame::NONE
                                .fill(badge_bg)
                                .corner_radius(CornerRadius::same(3))
                                .inner_margin(Margin::symmetric(6, 2))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(mode_name)
                                            .color(badge_fg)
                                            .strong()
                                            .size(11.0),
                                    );
                                });

                            if self.core.keisen_mode.is_active {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("[KEISEN]")
                                        .color(colors::ACCENT_CYAN)
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            if self.core.is_csv_active {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("[CSV]")
                                        .color(colors::ACCENT_GREEN)
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            if self.core.is_recording_macro {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("[REC]")
                                        .color(colors::ACCENT_RED)
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            ui.add_space(6.0);

                            let file_name = self
                                .core
                                .file_path
                                .as_ref()
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                                .unwrap_or("[No Name]");

                            let dirty_badge = if self.core.buffer.is_modified() {
                                " [+]"
                            } else {
                                ""
                            };
                            ui.label(
                                egui::RichText::new(format!("{}{}", file_name, dirty_badge))
                                    .color(colors::FG_TEXT)
                                    .strong(),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let lang_str = format!("{:?}", self.core.syntax_lang);
                                    ui.label(
                                        egui::RichText::new(lang_str)
                                            .color(colors::FG_DIMMED)
                                            .size(11.0),
                                    );
                                    ui.separator();

                                    let nl_name = format!("[{}]", self.core.newline.name());
                                    ui.label(
                                        egui::RichText::new(nl_name)
                                            .color(colors::FG_DIMMED)
                                            .size(11.0),
                                    );

                                    let enc_name = format!("[{}]", self.core.encoding.name());
                                    ui.label(
                                        egui::RichText::new(enc_name)
                                            .color(colors::FG_DIMMED)
                                            .size(11.0),
                                    );
                                    ui.separator();

                                    let cursor = self.core.cursor_pos();
                                    let pos_str =
                                        format!("Ln {}, Col {}", cursor.line + 1, cursor.col + 1);
                                    ui.label(
                                        egui::RichText::new(pos_str)
                                            .color(colors::FG_TEXT)
                                            .strong()
                                            .size(11.0),
                                    );
                                },
                            );
                        });
                    });

                // 下段: 最下段コマンドライン / メッセージ表示行（共通 App コアのパーサーバッファを直接描画）
                Frame::NONE
                    .fill(colors::BG_PANEL)
                    .inner_margin(Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if self.core.is_migemo_search {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "g/{}{}",
                                        self.core.migemo_query, self.ime_preedit
                                    ))
                                    .color(colors::ACCENT_YELLOW)
                                    .family(egui::FontFamily::Monospace),
                                );
                            } else if self.core.mode() == ViMode::CommandLine {
                                let cmd_str = self
                                    .core
                                    .vi
                                    .parser()
                                    .command_line_buffer()
                                    .map(|(prefix, buf)| {
                                        format!("{prefix}{buf}{}", self.ime_preedit)
                                    })
                                    .unwrap_or_else(|| ":".to_string());
                                ui.label(
                                    egui::RichText::new(cmd_str)
                                        .color(colors::FG_TEXT)
                                        .family(egui::FontFamily::Monospace),
                                );
                            } else if let Some(ref msg) = self.core.status_message {
                                let is_err = msg.contains("未保存")
                                    || msg.contains("エラー")
                                    || msg.contains("見つかりません")
                                    || msg.starts_with('E');
                                let color = if is_err {
                                    colors::ACCENT_RED
                                } else {
                                    colors::FG_TEXT
                                };
                                ui.label(egui::RichText::new(msg).color(color));
                            } else {
                                ui.label(
                                    egui::RichText::new("ready")
                                        .color(colors::FG_DIMMED)
                                        .size(11.0),
                                );
                            }
                        });
                    });
            });
    }
}
