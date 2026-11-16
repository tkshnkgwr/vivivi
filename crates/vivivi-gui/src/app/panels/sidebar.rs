//! 左サイドバー（ファイルツリー / アウトライン）描画モジュール。

use eframe::egui;
use egui::{Color32, Frame, Margin, Sense, Vec2};
use std::path::PathBuf;

use super::super::{SidebarTab, ViviviGuiApp};

/// サイドバーの 1 行（ファイルツリー／アウトライン共通）描画用パラメータ。
struct SidebarTreeRowSpec<'a> {
    depth: usize,
    expanded: Option<bool>,
    reserve_chevron_space: bool,
    icon: &'a str,
    label: &'a str,
    text_color: Color32,
    icon_color: Color32,
    is_active: bool,
}

/// Zed 風のサイドバー 1 行（全幅ホバーハイライト・アクティブ上下枠線・縦インデントガイド・指カーソル・全幅クリック）を描画します。
fn render_sidebar_tree_row(
    ui: &mut egui::Ui,
    pal: &crate::theme::GuiPalette,
    spec: SidebarTreeRowSpec<'_>,
) -> egui::Response {
    const ROW_HEIGHT: f32 = 22.0;
    const INDENT_STEP: f32 = 14.0;
    const LEFT_PAD: f32 = 6.0;
    const CHEVRON_SLOT: f32 = 14.0;

    let row_width = ui.available_width().max(1.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(row_width, ROW_HEIGHT), Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    let painter = ui.painter_at(rect);
    if spec.is_active {
        painter.rect_filled(rect, 0.0, pal.bg_selection);
        painter.hline(
            rect.left()..=rect.right(),
            rect.top() + 0.5,
            egui::Stroke::new(1.0_f32, pal.accent_blue),
        );
        painter.hline(
            rect.left()..=rect.right(),
            rect.bottom() - 0.5,
            egui::Stroke::new(1.0_f32, pal.accent_blue),
        );
    } else if response.hovered() {
        painter.rect_filled(rect, 0.0, pal.bg_hover);
    }

    // 階層インデントごとの縦ガイド線（Zed 風）
    let indent_base_x = rect.left() + LEFT_PAD;
    for lvl in 0..spec.depth {
        let gx = indent_base_x + (lvl as f32) * INDENT_STEP + 5.0;
        painter.line_segment(
            [
                egui::Pos2::new(gx, rect.top()),
                egui::Pos2::new(gx, rect.bottom()),
            ],
            egui::Stroke::new(1.0_f32, pal.border),
        );
    }

    let mut x = indent_base_x + (spec.depth as f32) * INDENT_STEP;
    let cy = rect.center().y;

    // フォルダ開閉シェブロン（v / >）または揃え用スロット
    if let Some(is_expanded) = spec.expanded {
        let cx = x + 5.0;
        let chevron_stroke = egui::Stroke::new(1.4_f32, pal.fg_dimmed);
        if is_expanded {
            painter.line_segment(
                [
                    egui::Pos2::new(cx - 3.0, cy - 1.5),
                    egui::Pos2::new(cx, cy + 2.0),
                ],
                chevron_stroke,
            );
            painter.line_segment(
                [
                    egui::Pos2::new(cx, cy + 2.0),
                    egui::Pos2::new(cx + 3.0, cy - 1.5),
                ],
                chevron_stroke,
            );
        } else {
            painter.line_segment(
                [
                    egui::Pos2::new(cx - 1.5, cy - 3.0),
                    egui::Pos2::new(cx + 2.0, cy),
                ],
                chevron_stroke,
            );
            painter.line_segment(
                [
                    egui::Pos2::new(cx + 2.0, cy),
                    egui::Pos2::new(cx - 1.5, cy + 3.0),
                ],
                chevron_stroke,
            );
        }
        x += CHEVRON_SLOT;
    } else if spec.reserve_chevron_space {
        x += CHEVRON_SLOT;
    }

    // アイコン描画
    if !spec.icon.is_empty() {
        let icon_galley = painter.layout_no_wrap(
            spec.icon.to_string(),
            egui::FontId::proportional(13.0),
            spec.icon_color,
        );
        let icon_pos = egui::Pos2::new(x, cy - icon_galley.size().y * 0.5);
        let icon_w = icon_galley.size().x;
        painter.galley(icon_pos, icon_galley, spec.icon_color);
        x += icon_w.max(15.0) + 5.0;
    }

    // ラベル描画
    let label_galley = painter.layout_no_wrap(
        spec.label.to_string(),
        egui::FontId::proportional(13.0),
        spec.text_color,
    );
    let label_pos = egui::Pos2::new(x, cy - label_galley.size().y * 0.5);
    painter.galley(label_pos, label_galley, spec.text_color);

    response
}

impl ViviviGuiApp {
    /// 2. ファイルツリー / アウトライン用折りたたみサイドバーの描画
    ///
    /// # 引数
    /// - `ctx`: 描画先の `egui::Context`
    pub fn render_sidebar(&mut self, ctx: &egui::Context) {
        let pal = self.palette();
        if !self.sidebar_open {
            return;
        }

        egui::SidePanel::left("sidebar_panel")
            .frame(Frame::NONE.fill(pal.bg_panel).inner_margin(Margin::same(6)))
            .default_width(self.sidebar_width)
            .width_range(160.0..=450.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.style_mut().interaction.selectable_labels = false;
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.sidebar_tab, SidebarTab::Files, " 📁 Files ");
                    if ui
                        .selectable_value(
                            &mut self.sidebar_tab,
                            SidebarTab::Outline,
                            " 🔖 Outline ",
                        )
                        .clicked()
                    {
                        self.core.sync_outline_if_dirty();
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| match self.sidebar_tab {
                    SidebarTab::Files => {
                        ui.label(
                            egui::RichText::new("PROJECT EXPLORER")
                                .color(pal.fg_dimmed)
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
                                .color(pal.fg_text),
                        );

                        let mut dir_to_navigate: Option<PathBuf> = None;
                        let mut dir_to_toggle: Option<PathBuf> = None;
                        let mut file_to_open: Option<PathBuf> = None;

                        ui.scope(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;

                            // 親ディレクトリ (..) への移動リンク
                            if let Some(parent) = self.current_dir.parent() {
                                let parent_path = parent.to_path_buf();
                                let up_resp = render_sidebar_tree_row(
                                    ui,
                                    &pal,
                                    SidebarTreeRowSpec {
                                        depth: 0,
                                        expanded: None,
                                        reserve_chevron_space: false,
                                        icon: "⬆",
                                        label: "..",
                                        text_color: pal.accent_blue,
                                        icon_color: pal.accent_blue,
                                        is_active: false,
                                    },
                                );
                                if up_resp.clicked() {
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
                                                .color(pal.accent_blue)
                                                .size(11.0),
                                        );
                                    });
                                    ui.add_space(2.0);
                                }
                                crate::app::async_tree::ScanStatus::Timeout => {
                                    ui.label(
                                        egui::RichText::new("⚠️ 読込タイムアウト (2.0s)")
                                            .color(pal.accent_yellow)
                                            .size(11.0),
                                    );
                                    ui.label(
                                        egui::RichText::new("クラウド/SMB共有が遅延しています")
                                            .color(pal.fg_dimmed)
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
                                            .color(pal.accent_red)
                                            .size(11.0),
                                    );
                                    if ui.button("🔄 再試行").clicked() {
                                        self.async_tree.request_scan(self.current_dir.clone());
                                    }
                                    ui.add_space(4.0);
                                }
                                _ => {}
                            }

                            let visible_rows = self.async_tree.visible_rows();
                            if !visible_rows.is_empty() {
                                for row in visible_rows {
                                    let icon = if row.is_dir {
                                        if row.is_expanded { "📂" } else { "📁" }
                                    } else if row.is_cloud_placeholder {
                                        "☁"
                                    } else if row.is_symlink {
                                        "🔗"
                                    } else {
                                        "📄"
                                    };
                                    let label = if row.is_loading {
                                        format!("{} (読込中...)", row.name)
                                    } else {
                                        row.name
                                    };
                                    let is_active = !row.is_dir
                                        && self.core.file_path.as_ref() == Some(&row.path);
                                    let row_resp = render_sidebar_tree_row(
                                        ui,
                                        &pal,
                                        SidebarTreeRowSpec {
                                            depth: row.depth,
                                            expanded: row.is_dir.then_some(row.is_expanded),
                                            reserve_chevron_space: true,
                                            icon,
                                            label: &label,
                                            text_color: pal.fg_text,
                                            icon_color: if row.is_dir {
                                                pal.accent_blue
                                            } else {
                                                pal.fg_dimmed
                                            },
                                            is_active,
                                        },
                                    );

                                    if row_resp.clicked() {
                                        if row.is_dir {
                                            dir_to_toggle = Some(row.path);
                                        } else {
                                            file_to_open = Some(row.path);
                                        }
                                    }
                                }
                            } else {
                                for path in &self.cached_files {
                                    let name =
                                        path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                                    let is_dir = path.is_dir();
                                    let icon = if is_dir { "📁" } else { "📄" };
                                    let is_active =
                                        !is_dir && self.core.file_path.as_ref() == Some(path);
                                    let row_resp = render_sidebar_tree_row(
                                        ui,
                                        &pal,
                                        SidebarTreeRowSpec {
                                            depth: 0,
                                            expanded: is_dir.then_some(false),
                                            reserve_chevron_space: true,
                                            icon,
                                            label: name,
                                            text_color: pal.fg_text,
                                            icon_color: if is_dir {
                                                pal.accent_blue
                                            } else {
                                                pal.fg_dimmed
                                            },
                                            is_active,
                                        },
                                    );

                                    if row_resp.clicked() {
                                        if is_dir {
                                            dir_to_navigate = Some(path.clone());
                                        } else {
                                            file_to_open = Some(path.clone());
                                        }
                                    }
                                }
                            }
                        });

                        if let Some(target_dir) = dir_to_toggle {
                            self.async_tree.toggle_expand(&target_dir);
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
                        self.core.sync_outline_if_dirty();
                        ui.label(
                            egui::RichText::new("SYMBOLS / OUTLINE")
                                .color(pal.fg_dimmed)
                                .size(11.0),
                        );
                        ui.add_space(4.0);

                        if self.core.outline_items.is_empty() {
                            ui.label(
                                egui::RichText::new("(No symbols detected)").color(pal.fg_dimmed),
                            );
                        } else {
                            let cursor_line = self.core.cursor.pos.line;
                            let active_outline_idx = self
                                .core
                                .outline_items
                                .iter()
                                .rposition(|item| item.line_idx <= cursor_line);
                            let mut jump_line = None;
                            ui.scope(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                for (idx, item) in self.core.outline_items.iter().enumerate() {
                                    let depth = item.level.saturating_sub(1);
                                    let (icon, icon_color) = match item.kind {
                                        vivivi_modes::outline::OutlineKind::Heading(_) => {
                                            ("#", pal.accent_blue)
                                        }
                                        vivivi_modes::outline::OutlineKind::Function => {
                                            ("fn", pal.accent_purple)
                                        }
                                        vivivi_modes::outline::OutlineKind::Struct => {
                                            ("S", pal.accent_yellow)
                                        }
                                        vivivi_modes::outline::OutlineKind::Enum => {
                                            ("E", pal.accent_yellow)
                                        }
                                        vivivi_modes::outline::OutlineKind::Class => {
                                            ("C", pal.accent_yellow)
                                        }
                                        vivivi_modes::outline::OutlineKind::Impl => {
                                            ("I", pal.accent_cyan)
                                        }
                                        vivivi_modes::outline::OutlineKind::Trait => {
                                            ("T", pal.accent_cyan)
                                        }
                                        vivivi_modes::outline::OutlineKind::Module => {
                                            ("m", pal.accent_green)
                                        }
                                        vivivi_modes::outline::OutlineKind::Section => {
                                            ("§", pal.fg_dimmed)
                                        }
                                    };
                                    let symbol_label =
                                        format!("L{}: {}", item.line_idx + 1, item.title);
                                    let row_resp = render_sidebar_tree_row(
                                        ui,
                                        &pal,
                                        SidebarTreeRowSpec {
                                            depth,
                                            expanded: None,
                                            reserve_chevron_space: false,
                                            icon,
                                            label: &symbol_label,
                                            text_color: pal.fg_text,
                                            icon_color,
                                            is_active: active_outline_idx == Some(idx),
                                        },
                                    );
                                    if row_resp.clicked() {
                                        jump_line = Some(item.line_idx);
                                    }
                                }
                            });
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
}
