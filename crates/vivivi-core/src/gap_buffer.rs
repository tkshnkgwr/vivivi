//! OpenViVi の `gap_buffer.h` に準拠したジェネリックな GapBuffer（ギャップバッファ）実装。
//!
//! ギャップバッファは、現在の編集カーソル位置に「空き領域（Gap）」を常駐させることで、
//! キーボード入力やバックスペース等の連続した局所的編集を $O(1)$（償却）で高速処理する
//! テキストエディタ専用の古典的かつ強力なデータ構造です。

/// ギャップバッファ構造体。
///
/// 内部の物理配列の中間に `gap_start` から `gap_len` 長の空き領域（ギャップ）を保持します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapBuffer<T> {
    /// 物理メモリ配列（データ前半部 + ギャップ + データ後半部）
    data: Vec<T>,
    /// ギャップの開始位置（論理インデックスと一致）
    gap_start: usize,
    /// ギャップの現在の空き容量
    gap_len: usize,
}

impl<T: Default + Clone> Default for GapBuffer<T> {
    fn default() -> Self {
        Self::with_capacity(32)
    }
}

impl<T: Default + Clone> GapBuffer<T> {
    /// デフォルト容量で空の新しい `GapBuffer` を生成します。
    pub fn new() -> Self {
        Self::default()
    }

    /// 指定した初期ギャップ容量を持つ新しい `GapBuffer` を生成します。
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = capacity.max(8);
        Self {
            data: vec![T::default(); cap],
            gap_start: 0,
            gap_len: cap,
        }
    }

    /// バッファに格納されている有効要素数（ギャップを除く）を返します。
    pub fn len(&self) -> usize {
        self.data.len() - self.gap_len
    }

    /// バッファが空かどうかを判定します。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 物理メモリ配列の総容量（データ要素数 + ギャップサイズ）を返します。
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    /// ギャップの現在の開始位置インデックスを返します。
    pub fn gap_start(&self) -> usize {
        self.gap_start
    }

    /// ギャップの現在の空き要素数を返します。
    pub fn gap_len(&self) -> usize {
        self.gap_len
    }

    /// 論理インデックスを物理配列のインデックスへ変換します。
    #[inline]
    fn to_physical_index(&self, index: usize) -> usize {
        if index < self.gap_start {
            index
        } else {
            index + self.gap_len
        }
    }

    /// ギャップを指定した論理インデックス `index` へ移動させます。
    pub fn move_gap(&mut self, index: usize) {
        let index = index.min(self.len());
        if index == self.gap_start {
            return;
        }

        if index < self.gap_start {
            // ギャップが左へ移動：[index, gap_start) の要素をギャップの右側へシフト
            let count = self.gap_start - index;
            for i in (0..count).rev() {
                let src = index + i;
                let dst = index + self.gap_len + i;
                self.data[dst] = std::mem::take(&mut self.data[src]);
            }
        } else {
            // ギャップが右へ移動：[gap_start + gap_len, index + gap_len) の要素をギャップの左側へシフト
            let count = index - self.gap_start;
            for i in 0..count {
                let src = self.gap_start + self.gap_len + i;
                let dst = self.gap_start + i;
                self.data[dst] = std::mem::take(&mut self.data[src]);
            }
        }

        self.gap_start = index;
    }

    /// ギャップの空き容量が少なくとも `required` 要素分あることを保証（必要に応じて自動拡張）します。
    pub fn reserve_gap(&mut self, required: usize) {
        if self.gap_len >= required {
            return;
        }

        let additional = required - self.gap_len;
        let new_gap_len = (self.data.len() + additional).max(self.data.len() * 2);
        let extra = new_gap_len - self.data.len();

        let old_len = self.data.len();
        self.data.resize(old_len + extra, T::default());

        // ギャップ後方のデータを新しいバッファの末尾側へシフト
        let post_gap_elements = old_len - (self.gap_start + self.gap_len);
        for i in (0..post_gap_elements).rev() {
            let src = self.gap_start + self.gap_len + i;
            let dst = self.gap_start + self.gap_len + extra + i;
            self.data[dst] = std::mem::take(&mut self.data[src]);
        }

        self.gap_len += extra;
    }

    /// 論理インデックス `index` にある要素の不変参照を取得します。
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(&self.data[self.to_physical_index(index)])
        } else {
            None
        }
    }

    /// 論理インデックス `index` にある要素の可変参照を取得します。
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() {
            let phys = self.to_physical_index(index);
            Some(&mut self.data[phys])
        } else {
            None
        }
    }

    /// 論理インデックス `index` に単一要素を挿入します。
    pub fn insert(&mut self, index: usize, val: T) {
        self.move_gap(index);
        self.reserve_gap(1);

        self.data[self.gap_start] = val;
        self.gap_start += 1;
        self.gap_len -= 1;
    }

    /// 論理インデックス `index` に要素のスライスを一括挿入します。
    pub fn insert_slice(&mut self, index: usize, slice: &[T]) {
        if slice.is_empty() {
            return;
        }
        self.move_gap(index);
        self.reserve_gap(slice.len());

        for (i, item) in slice.iter().enumerate() {
            self.data[self.gap_start + i] = item.clone();
        }
        self.gap_start += slice.len();
        self.gap_len -= slice.len();
    }

    /// 論理インデックス `index` の要素を削除して返します。
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }
        self.move_gap(index);
        let removed = std::mem::take(&mut self.data[self.gap_start + self.gap_len]);
        self.gap_len += 1;
        Some(removed)
    }

    /// 範囲 `[start, end)` の要素を一括削除します。
    pub fn remove_range(&mut self, start: usize, end: usize) {
        let start = start.min(self.len());
        let end = end.min(self.len());
        if start >= end {
            return;
        }

        self.move_gap(start);
        let count = end - start;
        for i in 0..count {
            self.data[self.gap_start + self.gap_len + i] = T::default();
        }
        self.gap_len += count;
    }

    /// バッファ内のすべての要素をクリアします。
    pub fn clear(&mut self) {
        let cap = self.data.len();
        self.data.fill_with(T::default);
        self.gap_start = 0;
        self.gap_len = cap;
    }

    /// バッファ内のすべての要素をイテレータとして返します。
    pub fn iter(&self) -> GapBufferIter<'_, T> {
        GapBufferIter {
            buffer: self,
            index: 0,
        }
    }
}

pub struct GapBufferIter<'a, T> {
    buffer: &'a GapBuffer<T>,
    index: usize,
}

impl<'a, T: Default + Clone> Iterator for GapBufferIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.buffer.get(self.index)?;
        self.index += 1;
        Some(item)
    }
}

impl std::fmt::Display for GapBuffer<char> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ch in self.iter() {
            write!(f, "{ch}")?;
        }
        Ok(())
    }
}

impl GapBuffer<char> {
    /// 文字列スライスから文字単位の GapBuffer を生成するヘルパーです。
    pub fn from_text(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        let mut buf = Self::with_capacity(chars.len() + 32);
        buf.insert_slice(0, &chars);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_buffer_basic() {
        let mut gb: GapBuffer<char> = GapBuffer::new();
        assert!(gb.is_empty());
        assert_eq!(gb.len(), 0);

        gb.insert(0, 'a');
        gb.insert(1, 'b');
        gb.insert(2, 'd');
        assert_eq!(gb.to_string(), "abd");

        gb.insert(2, 'c');
        assert_eq!(gb.to_string(), "abcd");

        let removed = gb.remove(1);
        assert_eq!(removed, Some('b'));
        assert_eq!(gb.to_string(), "acd");

        gb.remove_range(1, 3);
        assert_eq!(gb.to_string(), "a");
    }

    #[test]
    fn test_gap_buffer_slice_insertion_and_growth() {
        let mut gb = GapBuffer::from_text("Hello World!");
        assert_eq!(gb.to_string(), "Hello World!");

        let insert_text: Vec<char> = " beautiful".chars().collect();
        gb.insert_slice(5, &insert_text);
        assert_eq!(gb.to_string(), "Hello beautiful World!");

        // ギャップを先頭へ移動して挿入
        gb.insert(0, '>');
        assert_eq!(gb.to_string(), ">Hello beautiful World!");

        // ギャップを末尾へ移動して挿入
        let end = gb.len();
        gb.insert(end, '<');
        assert_eq!(gb.to_string(), ">Hello beautiful World!<");
    }
}
