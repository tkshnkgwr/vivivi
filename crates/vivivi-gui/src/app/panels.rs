//! メニューバー・マルチタブバー・左サイドバー・下部ステータスバー＆コマンドライン描画モジュール。

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, Margin, Sense, Vec2};
use std::path::PathBuf;
use vivivi_vi::ViMode;

use super::{SidebarTab, ViviviGuiApp};
use crate::theme::colors;

/// Zed 風のメニューポップアップ共通スタイル（最小幅・余白・セパレータ境界線）を適用します。
fn apply_menu_popup_style(ui: &mut egui::Ui, min_width: f32) {
    ui.set_min_width(min_width);
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
    ui.spacing_mut().button_padding = Vec2::new(8.0, 4.0);
    ui.visuals_mut().widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, Color32::from_rgb(45, 51, 74));
}

/// Zed 風の左寄せラベル＋右寄せショートカット付きメニュー項目を描画します。
fn menu_item(ui: &mut egui::Ui, label: &str, shortcut: &str) -> bool {
    let shortcut_color = Color32::from_rgb(125, 134, 172);
    let mut btn = egui::Button::new(label);
    if !shortcut.is_empty() {
        btn = btn.shortcut_text(egui::RichText::new(shortcut).color(shortcut_color));
    }
    ui.add(btn).clicked()
}

/// Zed 風のチェック状態付き左寄せラベル＋右寄せショートカット付きメニュー項目を描画します。
fn menu_toggle_item(ui: &mut egui::Ui, checked: &mut bool, label: &str, shortcut: &str) -> bool {
    let shortcut_color = Color32::from_rgb(125, 134, 172);
    let prefix = if *checked { "✓ " } else { "  " };
    let mut btn = egui::Button::new(format!("{prefix}{label}")).selected(*checked);
    if !shortcut.is_empty() {
        btn = btn.shortcut_text(egui::RichText::new(shortcut).color(shortcut_color));
    }
    if ui.add(btn).clicked() {
        *checked = !*checked;
        true
    } else {
        false
    }
}

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
                        apply_menu_popup_style(ui, 220.0);
                        if menu_item(ui, "New Tab", ":tabnew") {
                            self.core.new_tab(None);
                            ui.close_menu();
                        }
                        if menu_item(ui, "Save", ":w") {
                            self.core.execute_ex("w");
                            ui.close_menu();
                        }
                        if menu_item(ui, "Recent Files", ":recent") {
                            self.core.execute_ex("recent");
                            ui.close_menu();
                        }
                        if menu_item(ui, "2-Pane Filer", ":filer") {
                            self.core.execute_ex("filer");
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, "Close Tab", ":tabclose") {
                            let _ = self.core.close_tab();
                            ui.close_menu();
                        }
                        if menu_item(ui, "Quit", ":q") {
                            self.core.execute_ex("q");
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Edit", |ui| {
                        apply_menu_popup_style(ui, 240.0);
                        if menu_item(ui, "Undo", "u") {
                            self.core.execute_ex("undo");
                            ui.close_menu();
                        }
                        if menu_item(ui, "Redo", "ctrl-r") {
                            self.core.execute_ex("redo");
                            ui.close_menu();
                        }
                        ui.separator();
                        let mut keisen = self.core.keisen_mode.is_active;
                        if menu_toggle_item(ui, &mut keisen, "Toggle Keisen Mode", "f5") {
                            self.core.keisen_mode.toggle();
                            ui.close_menu();
                        }
                        let mut csv_mode = self.core.is_csv_active;
                        if menu_toggle_item(ui, &mut csv_mode, "Toggle CSV Mode", ":csv") {
                            self.core.execute_ex("csv");
                            ui.close_menu();
                        }
                        if menu_item(ui, "Clipboard History", "f6") {
                            self.core.show_clipboard_hist = !self.core.show_clipboard_hist;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, "Preferences...", "ctrl-,") {
                            self.settings_dialog.open(&self.core.config);
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("View", |ui| {
                        apply_menu_popup_style(ui, 250.0);
                        if menu_toggle_item(ui, &mut self.sidebar_open, "Sidebar", "ctrl-e") {
                            ui.close_menu();
                        }
                        if menu_toggle_item(
                            ui,
                            &mut self.core.show_visualize_graph,
                            "Visualize Project Graph",
                            ":graph",
                        ) {
                            ui.close_menu();
                        }
                        let mut hex_mode = self.core.is_hex_mode;
                        if menu_toggle_item(ui, &mut hex_mode, "Binary Mode (Hex Editor)", ":hex") {
                            self.core.toggle_hex_mode();
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_toggle_item(
                            ui,
                            &mut self.core.config.appearance.gui.show_minimap,
                            "Minimap",
                            ":minimap",
                        ) {
                            self.minimap_open = self.core.config.appearance.gui.show_minimap;
                            ui.close_menu();
                        }
                        if menu_toggle_item(
                            ui,
                            &mut self.core.show_line_numbers,
                            "Line Numbers",
                            ":set nu",
                        ) {
                            self.core.config.appearance.line_number = self.core.show_line_numbers;
                            ui.close_menu();
                        }
                        if menu_toggle_item(
                            ui,
                            &mut self.core.relative_number,
                            "Relative Line Numbers",
                            ":set rnu",
                        ) {
                            self.core.config.appearance.relative_number = self.core.relative_number;
                            ui.close_menu();
                        }
                        if menu_toggle_item(
                            ui,
                            &mut self.core.list_chars_config.enabled,
                            "Invisible Chars",
                            ":set list",
                        ) {
                            self.core.config.invisibles.enabled =
                                self.core.list_chars_config.enabled;
                            ui.close_menu();
                        }
                        if menu_toggle_item(
                            ui,
                            &mut self.core.config.appearance.gui.enable_gravity_drop,
                            "Delete Particles",
                            "",
                        ) {
                            self.particles.enabled =
                                self.core.config.appearance.gui.enable_gravity_drop;
                            ui.close_menu();
                        }

                        ui.separator();
                        ui.menu_button("Color Theme", |ui| {
                            apply_menu_popup_style(ui, 180.0);
                            let available = vivivi_app::theme::Theme::list_available_themes();
                            let current_name = self.active_theme.meta.name.to_lowercase();
                            for theme_name in available {
                                let is_active =
                                    current_name.contains(&theme_name.replace('-', " "));
                                if ui.selectable_label(is_active, &theme_name).clicked() {
                                    self.active_theme =
                                        vivivi_app::theme::Theme::load_by_name(&theme_name);
                                    crate::theme::apply_theme(ui.ctx(), &self.active_theme);
                                    self.core.config.appearance.theme = theme_name;
                                    self.syntax_cache.clear();
                                    ui.close_menu();
                                }
                            }
                        });
                    });

                    ui.menu_button("Window", |ui| {
                        apply_menu_popup_style(ui, 220.0);
                        if menu_item(ui, "Split Horizontal", ":sp") {
                            self.core.execute_ex("sp");
                            ui.close_menu();
                        }
                        if menu_item(ui, "Split Vertical", ":vsp") {
                            self.core.execute_ex("vsp");
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, "Close Window", ":close") {
                            self.core.execute_ex("close");
                            ui.close_menu();
                        }
                        if menu_item(ui, "Only Window", ":only") {
                            self.core.execute_ex("only");
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Tools", |ui| {
                        apply_menu_popup_style(ui, 260.0);
                        if menu_item(ui, "LSP Start", ":lsp start") {
                            self.core.start_lsp(None);
                            ui.close_menu();
                        }
                        if menu_item(ui, "LSP Stop", ":lsp stop") {
                            self.core.stop_lsp();
                            ui.close_menu();
                        }
                        if menu_item(ui, "LSP Hover", "K") {
                            self.core.request_lsp_hover();
                            ui.close_menu();
                        }
                        if menu_item(ui, "Go to Definition", "gd") {
                            self.core.request_lsp_definition();
                            ui.close_menu();
                        }
                        if menu_toggle_item(
                            ui,
                            &mut self.core.lsp_show_diagnostics_panel,
                            "LSP Diagnostics",
                            ":lsp diag",
                        ) {
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, "Expand Selection", "alt-up") {
                            self.core.expand_smart_selection();
                            ui.close_menu();
                        }
                        if menu_item(ui, "Shrink Selection", "alt-down") {
                            self.core.shrink_smart_selection();
                            ui.close_menu();
                        }
                        if menu_item(ui, "View Syntax Tree (AST)", ":ast") {
                            self.core.execute_ex("ast");
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, "Trim Trailing Whitespace", ":trim") {
                            self.core.execute_ex("trim");
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Help", |ui| {
                        apply_menu_popup_style(ui, 240.0);
                        if menu_item(ui, "Vi Cheatsheet", ":help") {
                            self.core.execute_ex("help");
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, "Check for Updates...", ":checkupdate") {
                            self.core.check_for_updates(true);
                            ui.close_menu();
                        }
                        if let moult_update::UpdateStatus::ReadyToApply { ref release, .. } =
                            self.core.update_status()
                            && menu_item(
                                ui,
                                &format!("Restart to Update (v{})", release.version),
                                ":restart",
                            )
                        {
                            let _ = self.core.apply_update_and_restart();
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.label(
                            egui::RichText::new("vivivi - Vi-like Editor in Rust")
                                .color(Color32::from_rgb(125, 134, 172))
                                .size(11.5),
                        );
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
                            self.core.config.appearance.gui.show_minimap = self.minimap_open;
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
                    ui.style_mut().interaction.selectable_labels = false;
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
                                nw: 2,
                                ne: 2,
                                sw: 0,
                                se: 0,
                            })
                            .inner_margin(Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let btn = ui
                                        .add(
                                            egui::Label::new(
                                                egui::RichText::new(tab_label)
                                                    .color(text_color)
                                                    .strong(),
                                            )
                                            .selectable(false)
                                            .sense(Sense::click()),
                                        )
                                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                                    if btn.clicked() {
                                        tab_to_switch = Some(idx);
                                    }

                                    if tab_count > 1 {
                                        let close_btn = ui
                                            .add(
                                                egui::Label::new(
                                                    egui::RichText::new("×")
                                                        .color(colors::FG_DIMMED)
                                                        .size(12.0),
                                                )
                                                .selectable(false)
                                                .sense(Sense::click()),
                                            )
                                            .on_hover_cursor(egui::CursorIcon::PointingHand);
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
                ui.style_mut().interaction.selectable_labels = false;
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
                            let up_label = ui
                                .add(
                                    egui::Label::new(
                                        egui::RichText::new("  ⬆ ..").color(colors::ACCENT_BLUE),
                                    )
                                    .selectable(false)
                                    .sense(Sense::click()),
                                )
                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                            if up_label.clicked() {
                                dir_to_navigate = Some(parent_path);
                            }
                        }

                        // 非同期走査ステータスに応じた耐障害表示
                        match &self.async_tree.status {
                            crate::app::async_tree::ScanStatus::Scanning { .. } => {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label(
                                        egui::RichText::new("読み込み中...")
                                            .color(colors::ACCENT_BLUE)
                                            .size(11.0),
                                    );
                                });
                                ui.add_space(2.0);
                            }
                            crate::app::async_tree::ScanStatus::Timeout => {
                                ui.label(
                                    egui::RichText::new("⚠️ 読込タイムアウト (2.0s)")
                                        .color(colors::ACCENT_YELLOW)
                                        .size(11.0),
                                );
                                ui.label(
                                    egui::RichText::new("クラウド/SMB共有が遅延しています")
                                        .color(colors::FG_DIMMED)
                                        .size(10.0),
                                );
                                if ui.button("🔄 再試行").clicked() {
                                    self.async_tree.request_scan(self.current_dir.clone());
                                }
                                ui.add_space(4.0);
                            }
                            crate::app::async_tree::ScanStatus::Error(err) => {
                                ui.label(
                                    egui::RichText::new(format!("⚠️ {err}"))
                                        .color(colors::ACCENT_RED)
                                        .size(11.0),
                                );
                                if ui.button("🔄 再試行").clicked() {
                                    self.async_tree.request_scan(self.current_dir.clone());
                                }
                                ui.add_space(4.0);
                            }
                            _ => {}
                        }

                        if !self.async_tree.entries.is_empty() {
                            for entry in &self.async_tree.entries {
                                let icon = if entry.is_dir {
                                    "📁"
                                } else if entry.is_cloud_placeholder {
                                    "☁"
                                } else if entry.is_symlink {
                                    "🔗"
                                } else {
                                    "📄"
                                };

                                let label_text = format!("  {} {}", icon, entry.name);
                                let label = ui
                                    .add(
                                        egui::Label::new(
                                            egui::RichText::new(label_text).color(colors::FG_TEXT),
                                        )
                                        .selectable(false)
                                        .sense(Sense::click()),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                                if label.double_clicked() || (entry.is_dir && label.clicked()) {
                                    if entry.is_dir {
                                        dir_to_navigate = Some(entry.path.clone());
                                    } else {
                                        file_to_open = Some(entry.path.clone());
                                    }
                                }
                            }
                        } else {
                            for path in &self.cached_files {
                                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                                let is_dir = path.is_dir();
                                let icon = if is_dir { "📁" } else { "📄" };
                                let label_text = format!("  {} {}", icon, name);

                                let label = ui
                                    .add(
                                        egui::Label::new(
                                            egui::RichText::new(label_text).color(colors::FG_TEXT),
                                        )
                                        .selectable(false)
                                        .sense(Sense::click()),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                                if label.double_clicked() || (is_dir && label.clicked()) {
                                    if is_dir {
                                        dir_to_navigate = Some(path.clone());
                                    } else {
                                        file_to_open = Some(path.clone());
                                    }
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
                                let label = ui
                                    .add(
                                        egui::Label::new(
                                            egui::RichText::new(symbol_label)
                                                .color(colors::FG_TEXT),
                                        )
                                        .selectable(false)
                                        .sense(Sense::click()),
                                    )
                                    .on_hover_cursor(egui::CursorIcon::PointingHand);
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
                                .corner_radius(CornerRadius::same(2))
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

                            if self.core.is_hex_mode {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("[HEX]")
                                        .color(colors::ACCENT_YELLOW)
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            if let Some(ref mgr) = self.core.lsp_manager {
                                let (errs, warns) = if let Some(ref path) = self.core.file_path {
                                    let uri = format!(
                                        "file:///{}",
                                        path.to_string_lossy().replace('\\', "/")
                                    );
                                    let diags = mgr.get_diagnostics(&uri).unwrap_or(&[]);
                                    let e = diags
                                        .iter()
                                        .filter(|d| {
                                            d.severity
                                                == Some(
                                                    vivivi_modes::lsp::DiagnosticSeverity::Error,
                                                )
                                        })
                                        .count();
                                    let w = diags
                                        .iter()
                                        .filter(|d| {
                                            d.severity
                                                == Some(
                                                    vivivi_modes::lsp::DiagnosticSeverity::Warning,
                                                )
                                        })
                                        .count();
                                    (e, w)
                                } else {
                                    (0, 0)
                                };
                                let lsp_text = if errs > 0 || warns > 0 {
                                    format!("[LSP E:{errs} W:{warns}]")
                                } else {
                                    "[LSP]".to_string()
                                };
                                let lsp_color = if errs > 0 {
                                    colors::ACCENT_RED
                                } else if warns > 0 {
                                    colors::ACCENT_YELLOW
                                } else {
                                    colors::ACCENT_CYAN
                                };
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(lsp_text)
                                        .color(lsp_color)
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
                                    // 自動アップデートバッジ (moult-update)
                                    match self.core.update_status() {
                                        moult_update::UpdateStatus::UpdateAvailable(ref rel) => {
                                            let label = format!("⬆ v{} 利用可能", rel.version);
                                            let btn = egui::Button::new(
                                                egui::RichText::new(label)
                                                    .color(colors::BG_PANEL)
                                                    .strong()
                                                    .size(11.0),
                                            )
                                            .fill(colors::ACCENT_CYAN);
                                            if ui
                                                .add(btn)
                                                .on_hover_text(
                                                    "クリックして新バージョンをダウンロード",
                                                )
                                                .clicked()
                                            {
                                                self.core.download_update();
                                            }
                                            ui.separator();
                                        }
                                        moult_update::UpdateStatus::Downloading {
                                            downloaded,
                                            total,
                                        } => {
                                            let text = if let Some(tot) = total {
                                                let pct = (downloaded * 100)
                                                    .checked_div(tot)
                                                    .unwrap_or(0);
                                                format!("🔄 DL中... {pct}%")
                                            } else {
                                                "🔄 DL中...".to_string()
                                            };
                                            ui.label(
                                                egui::RichText::new(text)
                                                    .color(colors::ACCENT_YELLOW)
                                                    .size(11.0),
                                            );
                                            ui.separator();
                                        }
                                        moult_update::UpdateStatus::ReadyToApply {
                                            ref release,
                                            ..
                                        } => {
                                            let label = format!(
                                                "✨ v{} 準備完了 [再起動]",
                                                release.version
                                            );
                                            let btn = egui::Button::new(
                                                egui::RichText::new(label)
                                                    .color(colors::BG_PANEL)
                                                    .strong()
                                                    .size(11.0),
                                            )
                                            .fill(colors::ACCENT_GREEN);
                                            if ui
                                                .add(btn)
                                                .on_hover_text(
                                                    "クリックして新バージョンを反映し再起動",
                                                )
                                                .clicked()
                                            {
                                                let _ = self.core.apply_update_and_restart();
                                            }
                                            ui.separator();
                                        }
                                        _ => {}
                                    }

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
