//! `vivivi-gui` デスクトップ GUI アプリケーションのエントリポイント。
//!
//! コマンドライン引数で指定されたファイルがあればエンコーディングと改行コードを自動判別して読み込み、
//! `eframe` ネイティブウィンドウ（Tokyo Night テーマ ＆ PlemolJP 優先フォント設定）を起動します。

#![windows_subsystem = "windows"]

use eframe::NativeOptions;
use std::path::PathBuf;
use vivivi_app::App;
use vivivi_core::{CharEncoding, NewlineType};
use vivivi_gui::{ViviviGuiApp, fonts, theme};

/// 公式アプリアイコン（256x256 raw RGBA）
const ICON_RGBA_256: &[u8] = include_bytes!("../../../assets/icons/icon_256.rgba");

fn main() -> eframe::Result {
    // 前回アップデート時に残存した旧バイナリ (*.old) を自動消去
    moult_update::cleanup_old_binaries();

    // コマンドライン引数から開くファイルパスを取得
    let args: Vec<String> = std::env::args().collect();
    let file_path = args.get(1).map(PathBuf::from);

    let (initial_text, file_path_opt, encoding, newline) = if let Some(ref path) = file_path {
        if path.is_file() {
            match std::fs::read(path) {
                Ok(bytes) => {
                    let (decoded_text, enc, nl) = vivivi_core::encoding::decode_bytes(&bytes);
                    (decoded_text, Some(path.clone()), enc, nl)
                }
                Err(_) => (
                    String::new(),
                    Some(path.clone()),
                    CharEncoding::Utf8,
                    NewlineType::Lf,
                ),
            }
        } else {
            (
                String::new(),
                Some(path.clone()),
                CharEncoding::Utf8,
                NewlineType::Lf,
            )
        }
    } else {
        (String::new(), None, CharEncoding::Utf8, NewlineType::Lf)
    };

    let core_app = App::new(&initial_text, file_path_opt, encoding, newline);

    // ウィンドウ表示状態を読み込み、マルチモニタ切断等の画面外脱出安全ガードを適用
    let window_state = vivivi_gui::WindowState::load_and_validate();

    let icon_data = egui::IconData {
        rgba: ICON_RGBA_256.to_vec(),
        width: 256,
        height: 256,
    };

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([window_state.width, window_state.height])
        .with_min_inner_size([
            vivivi_gui::window_state::MIN_WINDOW_WIDTH,
            vivivi_gui::window_state::MIN_WINDOW_HEIGHT,
        ])
        .with_active(true)
        .with_drag_and_drop(true)
        .with_icon(icon_data)
        .with_title("vivivi");

    if window_state.custom_position {
        viewport = viewport.with_position([window_state.x, window_state.y]);
    }
    if window_state.is_maximized {
        viewport = viewport.with_maximized(true);
    }

    let native_options = NativeOptions {
        viewport,
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "vivivi",
        native_options,
        Box::new(move |cc| {
            // スタイルとフォントの初期化
            let theme = vivivi_app::theme::Theme::load_by_name(&core_app.config.appearance.theme);
            theme::apply_theme(&cc.egui_ctx, &theme);
            fonts::setup_fonts(&cc.egui_ctx);

            let mut app = ViviviGuiApp::new(core_app, cc);
            app.active_theme = theme;
            app.window_state = window_state;
            Ok(Box::new(app))
        }),
    )
}
