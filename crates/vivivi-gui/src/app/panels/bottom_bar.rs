//! 下部ステータスバー ＆ 最下段コマンドライン描画モジュール。

use eframe::egui;
use egui::{CornerRadius, Frame, Margin};
use vivivi_vi::ViMode;

use super::super::ViviviGuiApp;

impl ViviviGuiApp {
    /// 3. ステータスバー ＆ 最下段コマンドラインの描画（`TextEdit` 不使用）
    pub(crate) fn render_bottom_panel(&mut self, ctx: &egui::Context) {
        let pal = self.palette();
        egui::TopBottomPanel::bottom("bottom_status_panel")
            .frame(Frame::NONE.fill(pal.bg_panel))
            .show(ctx, |ui| {
                // 上段: ステータスライン
                Frame::NONE
                    .fill(pal.bg_statusline)
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
                                        .color(pal.accent_cyan)
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            if self.core.is_csv_active {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("[CSV]")
                                        .color(pal.accent_green)
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            if self.core.is_hex_mode {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("[HEX]")
                                        .color(pal.accent_yellow)
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
                                    pal.accent_red
                                } else if warns > 0 {
                                    pal.accent_yellow
                                } else {
                                    pal.accent_cyan
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
                                        .color(pal.accent_red)
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
                                    .color(pal.fg_text)
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
                                                    .color(pal.bg_panel)
                                                    .strong()
                                                    .size(11.0),
                                            )
                                            .fill(pal.accent_cyan);
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
                                                    .color(pal.accent_yellow)
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
                                                    .color(pal.bg_panel)
                                                    .strong()
                                                    .size(11.0),
                                            )
                                            .fill(pal.accent_green);
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
                                            .color(pal.fg_dimmed)
                                            .size(11.0),
                                    );
                                    ui.separator();

                                    let nl_name = format!("[{}]", self.core.newline.name());
                                    ui.label(
                                        egui::RichText::new(nl_name)
                                            .color(pal.fg_dimmed)
                                            .size(11.0),
                                    );

                                    let enc_name = format!("[{}]", self.core.encoding.name());
                                    ui.label(
                                        egui::RichText::new(enc_name)
                                            .color(pal.fg_dimmed)
                                            .size(11.0),
                                    );
                                    ui.separator();

                                    let cursor = self.core.cursor_pos();
                                    let pos_str =
                                        format!("Ln {}, Col {}", cursor.line + 1, cursor.col + 1);
                                    ui.label(
                                        egui::RichText::new(pos_str)
                                            .color(pal.fg_text)
                                            .strong()
                                            .size(11.0),
                                    );
                                },
                            );
                        });
                    });

                // 下段: 最下段コマンドライン / メッセージ表示行（共通 App コアのパーサーバッファを直接描画）
                Frame::NONE
                    .fill(pal.bg_panel)
                    .inner_margin(Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if self.core.is_migemo_search {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "g/{}{}",
                                        self.core.migemo_query, self.ime_preedit
                                    ))
                                    .color(pal.accent_yellow)
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
                                        .color(pal.fg_text)
                                        .family(egui::FontFamily::Monospace),
                                );
                            } else if let Some(ref msg) = self.core.status_message {
                                let is_err = msg.contains("未保存")
                                    || msg.contains("エラー")
                                    || msg.contains("見つかりません")
                                    || msg.starts_with('E');
                                let color = if is_err { pal.accent_red } else { pal.fg_text };
                                ui.label(egui::RichText::new(msg).color(color));
                            } else {
                                ui.label(
                                    egui::RichText::new("ready").color(pal.fg_dimmed).size(11.0),
                                );
                            }
                        });
                    });
            });
    }
}
