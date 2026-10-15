//! OpenViVi の `gap_buffer.h` に準拠したジェネリックな GapBuffer（ギャップバッファ）実装。
//!
//! ギャップバッファは、現在の編集カーソル位置に「空き領域（Gap）」を常駐させることで、
//! キーボード入力やバックスペース等の連続した局所的編集を $O(1)$（償却）で高速処理する
//! テキストエディタ専用の古典的かつ強力なデータ構造です。

/// 局所的な挿入・削除を高速に行うジェネリックなギャップバッファ。
///
/// 物理配列の中間に「ギャップ（空き領域）」を維持し、カーソル周辺での連続した文字入力や削除を
/// $O(1)$（償却）で処理するテキストエディタ向けのコレクションです。
/// 要素型 `T` には `Default + Clone` を要求します。
///
/// # Examples
///
/// ```
/// use vivivi_core::GapBuffer;
///
/// let mut buf = GapBuffer::from_text("Hello");
/// buf.insert(5, '!');
/// assert_eq!(buf.to_string(), "Hello!");
/// ```
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
    /// デフォルトのギャップ容量（32要素）で空の新しい [`GapBuffer`] を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let buf: GapBuffer<char> = GapBuffer::new();
    /// assert!(buf.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 指定した初期ギャップ容量を持つ新しい [`GapBuffer`] を生成します。
    ///
    /// 最低でも 8 要素分の容量が自動的に確保されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let buf: GapBuffer<char> = GapBuffer::with_capacity(64);
    /// assert_eq!(buf.capacity(), 64);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = capacity.max(8);
        Self {
            data: vec![T::default(); cap],
            gap_start: 0,
            gap_len: cap,
        }
    }

    /// バッファ内に格納されている有効な要素数を返します。
    ///
    /// 物理配列長からギャップの長さを差し引いた論理要素数です。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let buf = GapBuffer::from_text("abc");
    /// assert_eq!(buf.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.data.len() - self.gap_len
    }

    /// バッファに有効な要素が存在しないかどうかを判定します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let buf: GapBuffer<char> = GapBuffer::new();
    /// assert!(buf.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 物理メモリ配列の総容量（有効データ数 + ギャップ空き容量）を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let buf: GapBuffer<char> = GapBuffer::with_capacity(16);
    /// assert_eq!(buf.capacity(), 16);
    /// ```
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    /// ギャップの現在の開始論理インデックスを返します。
    ///
    /// 要素の挿入や削除が直ちに行われるカーソル位置に相当します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let mut buf = GapBuffer::from_text("hello");
    /// buf.move_gap(2);
    /// assert_eq!(buf.gap_start(), 2);
    /// ```
    pub fn gap_start(&self) -> usize {
        self.gap_start
    }

    /// ギャップが現在保持している空き要素数を返します。
    ///
    /// この容量を超える挿入が行われると、自動的にメモリ拡張（再アロケーション）が発生します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let buf: GapBuffer<char> = GapBuffer::with_capacity(32);
    /// assert_eq!(buf.gap_len(), 32);
    /// ```
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
    ///
    /// 移動区間に存在する要素を物理配列内でシフトします。
    /// `index` が有効要素数を超える場合は自動的に末尾（`len()`）にクランプされます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let mut buf = GapBuffer::from_text("Hello");
    /// buf.move_gap(2);
    /// assert_eq!(buf.gap_start(), 2);
    /// ```
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
    ///
    /// 空き容量が不足している場合、メモリを拡張してギャップ後方の要素を末尾側へ退避させます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let mut buf: GapBuffer<char> = GapBuffer::with_capacity(8);
    /// buf.reserve_gap(16);
    /// assert!(buf.gap_len() >= 16);
    /// ```
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
    ///
    /// ギャップの空き領域を透過的にスキップして物理インデックスを参照します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。`index` が範囲外の場合は `None` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let buf = GapBuffer::from_text("Rust");
    /// assert_eq!(buf.get(1), Some(&'u'));
    /// assert_eq!(buf.get(10), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(&self.data[self.to_physical_index(index)])
        } else {
            None
        }
    }

    /// 論理インデックス `index` にある要素の可変参照を取得します。
    ///
    /// ギャップの空き領域を透過的にスキップして物理インデックスを参照します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。`index` が範囲外の場合は `None` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let mut buf = GapBuffer::from_text("Rust");
    /// if let Some(ch) = buf.get_mut(0) {
    ///     *ch = 'D';
    /// }
    /// assert_eq!(buf.to_string(), "Dust");
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() {
            let phys = self.to_physical_index(index);
            Some(&mut self.data[phys])
        } else {
            None
        }
    }

    /// 指定した論理インデックス `index` に単一要素を挿入します。
    ///
    /// ギャップを `index` へ移動させ、必要に応じて容量を自動拡張した上で要素を格納します。
    /// `index` が有効要素数を超える場合は末尾に挿入されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let mut buf = GapBuffer::from_text("ac");
    /// buf.insert(1, 'b');
    /// assert_eq!(buf.to_string(), "abc");
    /// ```
    pub fn insert(&mut self, index: usize, val: T) {
        self.move_gap(index);
        self.reserve_gap(1);

        self.data[self.gap_start] = val;
        self.gap_start += 1;
        self.gap_len -= 1;
    }

    /// 指定した論理インデックス `index` に要素のスライスを一括挿入します。
    ///
    /// ギャップの移動とメモリ拡張を1回で済ませるため、単一挿入を繰り返すより高速です。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let mut buf = GapBuffer::from_text("Hello!");
    /// let slice: Vec<char> = " World".chars().collect();
    /// buf.insert_slice(5, &slice);
    /// assert_eq!(buf.to_string(), "Hello World!");
    /// ```
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
    ///
    /// ギャップを `index` へ移動し、要素を取り出した上でギャップの長さを 1 増加させます。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。`index` が範囲外の場合は `None` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let mut buf = GapBuffer::from_text("abc");
    /// assert_eq!(buf.remove(1), Some('b'));
    /// assert_eq!(buf.to_string(), "ac");
    /// assert_eq!(buf.remove(5), None);
    /// ```
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
    ///
    /// `start` または `end` が `len()` を超える場合は自動的にクランプされます。
    /// `start >= end` の場合は何も行いません。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let mut buf = GapBuffer::from_text("Hello, World!");
    /// buf.remove_range(5, 12);
    /// assert_eq!(buf.to_string(), "Hello!");
    /// ```
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
    ///
    /// 物理メモリ配列の領域は解放せず、全領域を空きギャップとして初期化します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let mut buf = GapBuffer::from_text("Rust");
    /// buf.clear();
    /// assert!(buf.is_empty());
    /// assert_eq!(buf.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        let cap = self.data.len();
        self.data.fill_with(T::default);
        self.gap_start = 0;
        self.gap_len = cap;
    }

    /// バッファ内のすべての要素を先頭から順に走査するイテレータを返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let buf = GapBuffer::from_text("Vi");
    /// let s: String = buf.iter().collect();
    /// assert_eq!(s, "Vi");
    /// ```
    pub fn iter(&self) -> GapBufferIter<'_, T> {
        GapBufferIter {
            buffer: self,
            index: 0,
        }
    }
}

/// [`GapBuffer`] の全要素を順次参照するためのイテレータ構造体。
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
    /// 文字列スライスから文字単位の [`GapBuffer`] を生成します。
    ///
    /// 文字列の文字数に加え、初期編集用の空き容量（32文字分）があらかじめ確保されます。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_core::GapBuffer;
    ///
    /// let buf = GapBuffer::from_text("Hello ViVi");
    /// assert_eq!(buf.len(), 10);
    /// assert_eq!(buf.to_string(), "Hello ViVi");
    /// ```
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
    fn test_internal_gap_position_and_movement() {
        let mut gb: GapBuffer<char> = GapBuffer::new();
        // 初期状態のギャップ位置と長さ
        assert_eq!(gb.gap_start, 0);
        let initial_gap_len = gb.gap_len;
        assert!(initial_gap_len > 0);

        gb.insert(0, 'a');
        gb.insert(1, 'b');
        // 'a', 'b' を挿入後、gap_start は 2 に進む
        assert_eq!(gb.gap_start, 2);
        assert_eq!(gb.gap_len, initial_gap_len - 2);

        // ギャップを先頭 (0) へ移動
        gb.move_gap(0);
        assert_eq!(gb.gap_start, 0);

        // ギャップを末尾 (2) へ移動
        gb.move_gap(2);
        assert_eq!(gb.gap_start, 2);
    }
}
