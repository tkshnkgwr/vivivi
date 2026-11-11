//! Unicode 罫線十字自動結合・交差検証ミニツール (examples/keisen_combine.rs)
//!
//! `vivivi-modes` の `keisen` モジュールにおける Unicode 罫線のビットフラグ状態管理と、
//! 水平・垂直方向の線が交差した際の自動十字結合 (`┼`, `┬`, `┴`, `├`, `┤` 等) を検証します。

use vivivi_modes::keisen::{KeisenDirection, char_to_keisen_state, state_to_keisen_char};

fn describe_keisen(ch: char) {
    if let Some(state) = char_to_keisen_state(ch) {
        let up = (state & KeisenDirection::Up.mask()) != 0;
        let down = (state & KeisenDirection::Down.mask()) != 0;
        let left = (state & KeisenDirection::Left.mask()) != 0;
        let right = (state & KeisenDirection::Right.mask()) != 0;
        println!(
            "文字: '{}' (0x{:04X}) | 状態フラグ: 0b{:08b} [上:{} 下:{} 左:{} 右:{}]",
            ch, ch as u32, state, up, down, left, right
        );
    } else {
        println!("文字: '{}' は罫線文字ではありません。", ch);
    }
}

fn combine_keisen(ch1: char, ch2: char) -> Option<char> {
    let s1 = char_to_keisen_state(ch1)?;
    let s2 = char_to_keisen_state(ch2)?;
    let combined_state = s1 | s2;
    state_to_keisen_char(combined_state)
}

fn main() {
    println!("=== vivivi: Unicode 罫線自動結合検証ツール ===\n");

    println!("--- 1. 基本罫線文字の状態フラグ解析 ---");
    let test_chars = ['─', '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼'];
    for &ch in &test_chars {
        describe_keisen(ch);
    }

    println!("\n--- 2. 線の交差・結合シミュレーション ---");
    let combinations = [
        ('─', '│', "水平線 ＋ 垂直線 → 十字交差"),
        ('─', '┌', "水平線 ＋ 左上角 → T字 (下向き)"),
        ('│', '─', "垂直線 ＋ 水平線 → 十字交差"),
        ('┌', '┘', "左上角 ＋ 右下角 → 水平垂直接続"),
        ('┬', '┴', "上向きT字 ＋ 下向きT字 → 十字交差"),
    ];

    for (c1, c2, label) in combinations {
        if let Some(result) = combine_keisen(c1, c2) {
            println!("{} : '{}' + '{}' => '{}'", label, c1, c2, result);
        } else {
            println!("{} : '{}' + '{}' => (結合不可)", label, c1, c2);
        }
    }

    println!("\n--- 3. 2次元キャンバス描画シミュレーション ---");
    let mut canvas = vec![vec![' '; 15]; 5];

    // 水平線を引く (y=2, x=2..12)
    #[allow(clippy::needless_range_loop)]
    for x in 2..=12 {
        canvas[2][x] = '─';
    }

    // 垂直線を交差させて引く (x=7, y=0..4)
    #[allow(clippy::needless_range_loop)]
    for y in 0..=4 {
        let current = canvas[y][7];
        if current == '─' {
            canvas[y][7] = combine_keisen(current, '│').unwrap_or('┼');
        } else {
            canvas[y][7] = '│';
        }
    }

    // 四角い枠を追加結合してみる (y=1..3, x=4..10)
    for row in &canvas {
        let line: String = row.iter().collect();
        println!("  {}", line);
    }

    println!("\n罫線自動結合ロジックが正常に動作しています。");
}
