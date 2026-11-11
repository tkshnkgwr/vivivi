//! GapBuffer 局所編集デモ (examples/gap_buffer_demo.rs)
//!
//! `vivivi-core` の `GapBuffer` における O(1) 局所編集と、カーソル移動に伴う
//! メモリ内ギャップ移動の挙動を標準出力で視覚的に可視化します。

use vivivi_core::GapBuffer;

fn print_gap_state(step_name: &str, gb: &GapBuffer<char>) {
    let start = gb.gap_start();
    let gap_len = gb.gap_len();
    let total_len = gb.len();
    let cap = gb.capacity();

    let text: String = gb.iter().collect();

    println!("============================================================");
    println!("Step: {}", step_name);
    println!("------------------------------------------------------------");
    println!("論理テキスト: \"{}\"", text);
    println!(
        "文字数 (len): {} | ギャップ長: {} | 総容量 (cap): {}",
        total_len, gap_len, cap
    );
    println!("ギャップ開始位置 (カーソル): {}", start);

    // メモリレイアウトの可視化
    print!("メモリ配置: [");
    // GapBuffer の iter() はギャップをスキップした論理要素を返すため、
    // ギャップ前とギャップ後を分けて表示する
    for i in 0..start {
        if let Some(&ch) = gb.get(i) {
            print!("{}", ch);
        }
    }
    print!(" <--- GAP ({} bytes) ---> ", gap_len);
    for i in start..total_len {
        if let Some(&ch) = gb.get(i) {
            print!("{}", ch);
        }
    }
    println!("]");
    println!();
}

fn main() {
    println!("=== vivivi: GapBuffer 可視化デモ ===");
    println!("カーソル周辺の局所編集を O(1) で行う GapBuffer の内部メモリ挙動を確認します。\n");

    let mut gb = GapBuffer::with_capacity(16);
    print_gap_state("初期状態 (空バッファ, capacity=16)", &gb);

    // 1. "Hello" を挿入
    let hello: Vec<char> = "Hello".chars().collect();
    gb.insert_slice(0, &hello);
    print_gap_state("\"Hello\" を先頭に挿入", &gb);

    // 2. 末尾に " World!" を挿入
    let world: Vec<char> = " World!".chars().collect();
    let len = gb.len();
    gb.insert_slice(len, &world);
    print_gap_state("末尾に \" World!\" を追加挿入", &gb);

    // 3. カーソルを "Hello" と " World!" の間 (インデックス 5) に移動
    gb.move_gap(5);
    print_gap_state(
        "カーソル(ギャップ)をインデックス 5 ('Hello' の直後) へ移動",
        &gb,
    );

    // 4. その位置に " Beautiful" を挿入
    let beautiful: Vec<char> = " Beautiful".chars().collect();
    gb.insert_slice(5, &beautiful);
    print_gap_state("インデックス 5 に \" Beautiful\" を即座に挿入 (O(1))", &gb);

    // 5. 1文字削除
    gb.remove(5);
    print_gap_state("インデックス 5 の文字 (' ') を削除", &gb);

    println!(
        "デモが正常に完了しました。GapBuffer によりカーソル位置での編集が効率的に行われます。"
    );
}
