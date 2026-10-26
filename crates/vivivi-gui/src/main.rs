//! `vivivi-gui` デスクトップ GUI アプリケーションのエントリポイント。
//!
//! コマンドライン引数で指定されたファイルがあればエンコーディングと改行コードを自動判別して読み込み、
//! `eframe` ネイティブウィンドウ（Tokyo Night テーマ ＆ PlemolJP 優先フォント設定）を起動します。

use eframe::NativeOptions;
use std::path::PathBuf;
use vivivi_app::App;
use vivivi_core::{CharEncoding, NewlineType};
use vivivi_gui::{ViviviGuiApp, fonts, theme};

fn main() -> eframe::Result {
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

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("vivivi"),
        ..Default::default()
    };

    eframe::run_native(
        "vivivi",
        native_options,
        Box::new(|cc| {
            // ステップ 1: スタイルとフォントの初期化
            theme::setup_visuals(&cc.egui_ctx);
            fonts::setup_fonts(&cc.egui_ctx);

            Ok(Box::new(ViviviGuiApp::new(core_app, cc)))
        }),
    )
}
