//! コード折り畳み（Code Folding）マネージャ。
//!
//! ViVi 3 / ViVi 4 の折り畳み機能（インデント階層・括弧ブロック）に基づき、
//! 行の折り畳み範囲検出、折り畳み/展開状態管理、
//! および表示行マッピング・プレビュー文字列生成を提供します。

/// 折り畳み範囲。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldRange {
    /// 開始行（1-indexed, 折り畳み時もプレビュー行として表示される行）
    pub start_line: usize,
    /// 終了行（1-indexed, この行までが折り畳まれる）
    pub end_line: usize,
    /// インデント深度・ネストレベル
    pub level: usize,
    /// 現在展開されているかどうか（true: 開いている, false: 折り畳まれている）
    pub is_open: bool,
}

impl FoldRange {
    /// 指定行がこの折り畳み範囲内に含まれているかを返します。
    pub fn contains(&self, line: usize) -> bool {
        line >= self.start_line && line <= self.end_line
    }

    /// 折り畳まれた結果、隠される行（開始行以外の行）かを判定します。
    pub fn hides(&self, line: usize) -> bool {
        !self.is_open && line > self.start_line && line <= self.end_line
    }
}

/// 折り畳みマネージャ本体。
#[derive(Debug, Clone, Default)]
pub struct FoldManager {
    pub folds: Vec<FoldRange>,
}

impl FoldManager {
    /// 新しい `FoldManager` を生成します。
    pub fn new() -> Self {
        Self::default()
    }

    /// インデント階層に基づいてテキストから折り畳みブロックを自動検出します。
    pub fn compute_indent_folds(&mut self, text: &str, tab_width: usize) {
        self.folds.clear();
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 2 {
            return;
        }

        // 各行のインデント幅（半角空白換算）を計算
        let indents: Vec<usize> = lines
            .iter()
            .map(|l| {
                let mut count = 0;
                for c in l.chars() {
                    if c == ' ' {
                        count += 1;
                    } else if c == '\t' {
                        count += tab_width;
                    } else {
                        break;
                    }
                }
                count
            })
            .collect();

        let mut stack: Vec<(usize, usize)> = Vec::new(); // (line_idx, indent)

        for i in 0..lines.len() {
            let line = lines[i];
            if line.trim().is_empty() {
                continue;
            }

            let cur_indent = indents[i];

            while let Some(&(top_line, top_indent)) = stack.last() {
                if cur_indent <= top_indent {
                    // ブロックの終端
                    if i > top_line + 1 {
                        self.folds.push(FoldRange {
                            start_line: top_line + 1,
                            end_line: i, // 空行を除く手前
                            level: top_indent / tab_width + 1,
                            is_open: true,
                        });
                    }
                    stack.pop();
                } else {
                    break;
                }
            }

            // 次の行が存在し、次の行が自分より深いインデントならスタックに積む
            let mut next_idx = i + 1;
            while next_idx < lines.len() && lines[next_idx].trim().is_empty() {
                next_idx += 1;
            }
            if next_idx < lines.len() && indents[next_idx] > cur_indent {
                stack.push((i, cur_indent));
            }
        }

        // 残りのスタックを処理
        let last_line = lines.len();
        while let Some((top_line, top_indent)) = stack.pop() {
            if last_line > top_line + 1 {
                self.folds.push(FoldRange {
                    start_line: top_line + 1,
                    end_line: last_line,
                    level: top_indent / tab_width + 1,
                    is_open: true,
                });
            }
        }

        // 開始行昇順にソート
        self.folds.sort_by_key(|f| (f.start_line, f.level));
    }

    /// カーソル行を含む最も内側の折り畳みをトグルします (`za`)。
    pub fn toggle_fold(&mut self, line: usize) -> bool {
        if let Some(fold) = self.folds.iter_mut().rev().find(|f| f.contains(line)) {
            fold.is_open = !fold.is_open;
            true
        } else {
            false
        }
    }

    /// カーソル行を含む最も内側の折り畳みを閉じます (`zc`)。
    pub fn close_fold(&mut self, line: usize) -> bool {
        if let Some(fold) = self.folds.iter_mut().rev().find(|f| f.contains(line)) {
            fold.is_open = false;
            true
        } else {
            false
        }
    }

    /// カーソル行を含む最も内側の折り畳みを開きます (`zo`)。
    pub fn open_fold(&mut self, line: usize) -> bool {
        if let Some(fold) = self.folds.iter_mut().rev().find(|f| f.contains(line)) {
            fold.is_open = true;
            true
        } else {
            false
        }
    }

    /// すべての折り畳みを閉じます (`zM`)。
    pub fn close_all(&mut self) {
        for f in &mut self.folds {
            f.is_open = false;
        }
    }

    /// すべての折り畳みを開きます (`zR`)。
    pub fn open_all(&mut self) {
        for f in &mut self.folds {
            f.is_open = true;
        }
    }

    /// 指定した深さレベル以上の折り畳みを閉じます (`z<lvl>`)。
    pub fn close_to_level(&mut self, level: usize) {
        for f in &mut self.folds {
            f.is_open = f.level < level;
        }
    }

    /// 指定行が他の折り畳みによって非表示（隠蔽）されているかを返します。
    pub fn is_line_hidden(&self, line: usize) -> bool {
        self.folds.iter().any(|f| f.hides(line))
    }

    /// 指定行が閉じられた折り畳みの開始行である場合、プレースホルダー文字列を返します。
    pub fn get_fold_placeholder(&self, line: usize, first_line_text: &str) -> Option<String> {
        let fold = self
            .folds
            .iter()
            .find(|f| !f.is_open && f.start_line == line)?;
        let hidden_count = fold.end_line - fold.start_line + 1;
        Some(format!(
            "+--- {:3} lines: {} ---",
            hidden_count,
            first_line_text.trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_indent_folds() {
        let code = "fn main() {\n    let a = 1;\n    let b = 2;\n    println!(\"{}\", a + b);\n}\n";
        let mut fm = FoldManager::new();
        fm.compute_indent_folds(code, 4);

        assert!(!fm.folds.is_empty());
        let top_fold = &fm.folds[0];
        assert_eq!(top_fold.start_line, 1);
        assert_eq!(top_fold.end_line, 4);

        // トグルテスト
        assert!(fm.toggle_fold(2));
        assert!(!fm.folds[0].is_open);
        assert!(fm.is_line_hidden(2));
        assert!(fm.is_line_hidden(3));
        assert!(!fm.is_line_hidden(1)); // 開始行は隠れない
    }
}
