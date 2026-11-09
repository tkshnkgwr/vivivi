//! `vivivi-gui` ビルドスクリプト。
//!
//! Windows ビルド時に `assets/icons/icon.ico` を実行バイナリのリソースとして静的埋め込みします。

fn main() {
    #[cfg(windows)]
    {
        // 開発ビルドまたはリリースビルド時に Windows PE アイコンを設定
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/icons/icon.ico");
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Failed to compile Windows resource icon: {e}");
        }
    }
}
