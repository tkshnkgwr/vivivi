//! メニューバー ＆ マルチタブバー描画モジュール。

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, Margin, Sense, Vec2};

use super::super::{SidebarTab, ViviviGuiApp};
use super::{apply_menu_popup_style, menu_item, menu_toggle_item};

impl ViviviGuiApp {
    /// 1. メニューバー ＆ マルチタブバーの描画
    pub(crate) fn render_top_panel(&mut self, ctx: &egui::Context) {
        self.core.sync_active_tab_state();
        let pal = self.palette();

        egui::TopBottomPanel::top("top_panel")
            .frame(Frame::NONE.fill(pal.bg_panel))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                    ui.visuals_mut().widgets.hovered.bg_fill = pal.bg_hover;

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
                        let mut outline_open =
                            self.sidebar_open && self.sidebar_tab == SidebarTab::Outline;
                        if menu_toggle_item(ui, &mut outline_open, "Outline", "f8") {
                            self.core.toggle_outline();
                            self.sync_sidebar_requests();
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
                        if menu_toggle_item(ui, &mut self.core.wrap, "Word Wrap", ":set wrap") {
                            self.core.config.appearance.wrap = self.core.wrap;
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
                            for theme_name in available {
                                let is_active = Self::theme_name_matches(
                                    &self.active_theme.meta.name,
                                    &theme_name,
                                );
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
                        if menu_item(ui, "External Command...", ":cmd") {
                            self.core.open_cmd_exec_dialog();
                            ui.close_menu();
                        }
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
                                .color(pal.fg_dimmed)
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
                            pal.bg_editor
                        } else {
                            pal.bg_panel
                        };

                        let text_color = if is_active {
                            pal.fg_text
                        } else {
                            pal.fg_dimmed
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
                                                        .color(pal.fg_dimmed)
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
                        egui::Button::new(egui::RichText::new(" + ").color(pal.fg_dimmed))
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
}
