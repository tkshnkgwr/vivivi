//! 設定ダイアログ後半カテゴリ（4. 入力・編集 / 5. 独自ツール / 6. キーマップ）描画モジュール。

use eframe::egui;

use super::super::ViviviGuiApp;

impl ViviviGuiApp {
    /// 4. 入力・編集 (Editing)
    pub(super) fn render_category_editing(&mut self, ui: &mut egui::Ui) {
        let pal = self.palette();
        ui.heading("✏ 4. 入力・編集設定 (Editing)");
        ui.label(
            egui::RichText::new("タブ幅、インデント、および検索のマッチング挙動を設定します。")
                .color(pal.fg_dimmed),
        );
        ui.separator();

        ui.spacing_mut().slider_width = 180.0;
        egui::Grid::new("editing_grid")
            .num_columns(2)
            .spacing([24.0, 10.0])
            .show(ui, |ui| {
                Self::nowrap_label(ui, "タブ幅 (Tab Stop):");
                ui.add(
                    egui::Slider::new(
                        &mut self.settings_dialog.draft_config.editing.tab_width,
                        1..=16,
                    )
                    .text("マス"),
                );
                ui.end_row();
            });

        ui.add_space(10.0);
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.expand_tab,
            "タブ文字入力を半角スペースに展開する (Expand Tab)",
        );
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.keisen_protect,
            "罫線保護編集を行う (枠線内の文字挿入・削除時に自動空白伸縮)",
        );
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.match_paren,
            "カーソル位置の対応括弧を強調表示する (%)",
        );
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.dabbrev_ignorecase,
            "動的補完 (dabbrev / Ctrl-L, Ctrl-M) で大文字小文字を同一視する",
        );
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.trim_on_save,
            "ファイル保存時に行末の空白を自動削除する (trim_on_save / :set trim)",
        );

        ui.add_space(5.0);
        ui.separator();
        ui.label(egui::RichText::new("【検索オプション】").strong());
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.ignorecase,
            "検索時に英大文字・小文字を同一視する (ignorecase / :set ic)",
        );
        ui.checkbox(
            &mut self.settings_dialog.draft_config.editing.smartcase,
            "大文字が含まれる場合のみ大文字・小文字を厳密に区別する (smartcase / :set scs)",
        );
    }

    /// 5. 独自ツール (Tools)
    pub(super) fn render_category_tools(&mut self, ui: &mut egui::Ui) {
        let pal = self.palette();
        ui.heading("⚒ 5. 独自ツール設定 (Tools / OpenViVi)");
        ui.label(
            egui::RichText::new("OpenViVi 特有のビジュアルギミックや入力支援ツールを設定します。")
                .color(pal.fg_dimmed),
        );
        ui.separator();

        ui.checkbox(
            &mut self
                .settings_dialog
                .draft_config
                .appearance
                .gui
                .enable_gravity_drop,
            "文字削除時の物理重力落下アニメーション (Particles / OpenViVi演出)",
        );
        ui.label(
            egui::RichText::new("  ※Normalモードでの 'x' や削除時に、文字が物理演算で落下します。")
                .color(pal.fg_dimmed),
        );

        ui.add_space(5.0);
        ui.checkbox(
            &mut self
                .settings_dialog
                .draft_config
                .appearance
                .gui
                .reactive_idle_sleep,
            "アイドル時 CPU 0% 省電力リアクティブ制御 (Reactive Sleep)",
        );
        ui.label(
            egui::RichText::new(
                "  ※キー入力やアニメーションがない待機時のCPU消費をゼロに抑えます。",
            )
            .color(pal.fg_dimmed),
        );

        ui.add_space(5.0);
        ui.separator();
        ui.label(egui::RichText::new("【インテリジェント入力支援機能】").strong());
        ui.label("• ローマ字日本語検索 (Migemo): 'g/' キーでローマ字のまま漢字バッファを検索");
        ui.label("• ダイナミックマクロ (Dynamic Macro): 'Alt-M' で直前の繰り返し操作を自動再現");
        ui.label("• 字句編集予測 (LexEdit): 'Ctrl-J' でバッファ内の文脈から次の入力を自動予測");
        ui.label("• 静的略語展開 (Abbreviation): 'Ctrl-;' で登録略語を展開");
        ui.label("• Zen-Coding (Emmet): HTML/XMLタグを 'Ctrl-,' (Alt-, / :zen) で即時展開");
    }

    /// 6. キーマップ (Keymap)
    pub(super) fn render_category_keymap(&mut self, ui: &mut egui::Ui) {
        let pal = self.palette();
        ui.heading("⌨ 6. キーマップ・ショートカット (Keymap)");
        ui.label(
            egui::RichText::new("主要な操作モードおよびキーバインドの確認リファレンスです。")
                .color(pal.fg_dimmed),
        );
        ui.separator();

        egui::Grid::new("keymap_reference_grid")
            .striped(true)
            .num_columns(3)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                Self::nowrap_label(ui, egui::RichText::new("モード / キー").strong());
                Self::nowrap_label(ui, egui::RichText::new("機能・コマンド").strong());
                Self::nowrap_label(ui, egui::RichText::new("説明").strong());
                ui.end_row();

                let keymaps = [
                    ("Normal", "h, j, k, l", "左・下・上・右カーソル移動"),
                    (
                        "Normal",
                        "i, a, o",
                        "Insertモード移行（カーソル位置・直後・次行）",
                    ),
                    (
                        "Normal",
                        "v, V, Ctrl-v",
                        "Visualモード（文字・行・矩形選択）",
                    ),
                    ("Normal", ":", "Ex コマンドライン入力へ移行"),
                    (
                        "Normal",
                        "u / Ctrl-r",
                        "アンドゥ（元に戻す） / リドゥ（やり直し）",
                    ),
                    ("Normal", "x", "1文字削除（重力落下アニメーション）"),
                    ("Normal", "g/", "Migemo ローマ字日本語検索"),
                    ("Normal", "F1 / :help", "オンラインヘルプビューア起動"),
                    ("Normal", "F6", "クリップボード履歴モーダル表示"),
                    ("Normal", ":filer", "2ペイン・ファイラー起動"),
                    ("Normal", ":preferences", "設定ダイアログ表示 (Ctrl+,)"),
                    ("GUI", "Ctrl+,", "設定ダイアログを開く"),
                    (
                        "GUI",
                        "Ctrl+n / Ctrl+w",
                        "新規タブ作成 / カレントタブを閉じる",
                    ),
                    ("Insert", "Ctrl-[", "Normal モードへ復帰 (Esc互換)"),
                    ("Insert", "Ctrl-j", "字句編集予測 (LexEdit)"),
                    ("Insert", "Ctrl-;", "静的略語展開 (Abbrev)"),
                ];

                for (mode, keys, desc) in keymaps {
                    Self::nowrap_label(ui, mode);
                    Self::nowrap_label(
                        ui,
                        egui::RichText::new(keys)
                            .family(egui::FontFamily::Monospace)
                            .color(pal.accent_cyan),
                    );
                    ui.label(desc);
                    ui.end_row();
                }
            });
    }
}
