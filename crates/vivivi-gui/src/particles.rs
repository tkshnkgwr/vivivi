//! 削除文字の落下物理アニメーション ＆ 省電力ハイブリッド制御モジュール（ステップ 4）。
//!
//! `x`, `dd`, `dw`, `Backspace` 等で文字が削除された際、該当グリフのピクセル座標から
//! 2D 物理パーティクルを生成して放物線落下＆フェードアウト（0.5秒）させます。
//! パーティクルが存在する間のみ `ctx.request_repaint()` を呼び出し、消滅後は
//! `egui` のリアクティブモード（入力待機時 CPU 0%）を完全に維持します。

use egui::{Align2, Color32, FontId, Painter, Pos2, Vec2};

/// 同時に保持するパーティクルの最大数（大量行削除時のフレームレート保護）
const MAX_PARTICLES: usize = 256;

/// デフォルトのフェードアウト寿命（秒）
pub const DEFAULT_LIFETIME: f32 = 0.5;

/// デフォルトの重力加速度（px/s²）
pub const DEFAULT_GRAVITY: f32 = 720.0;

/// 落下アニメーションする単一文字パーティクル。
///
/// # Examples
/// ```
/// use egui::{Color32, Pos2, Vec2};
/// use vivivi_gui::Particle;
///
/// let p = Particle {
///     ch: 'x',
///     pos: Pos2::new(10.0, 20.0),
///     vel: Vec2::new(0.0, -50.0),
///     color: Color32::RED,
///     elapsed: 0.25,
///     lifetime: 0.5,
/// };
/// assert!((p.life_ratio() - 0.5).abs() < 1e-5);
/// ```
#[derive(Debug, Clone)]
pub struct Particle {
    /// 削除された文字グリフ
    pub ch: char,
    /// 現在のスクリーン座標 (px)
    pub pos: Pos2,
    /// 現在の速度ベクトル (px/s)
    pub vel: Vec2,
    /// ベースカラー
    pub color: Color32,
    /// 生成からの経過時間 (s)
    pub elapsed: f32,
    /// 消滅までの寿命 (s)
    pub lifetime: f32,
}

impl Particle {
    /// 現在の残り寿命比率（`1.0` = 生成直後、`0.0` = 消滅）を返します。
    ///
    /// `lifetime` が `0.0` 以下の場合はパニックせず `0.0` を返します。
    ///
    /// # Examples
    /// ```
    /// use egui::{Color32, Pos2, Vec2};
    /// use vivivi_gui::Particle;
    ///
    /// let p = Particle {
    ///     ch: 'A',
    ///     pos: Pos2::ZERO,
    ///     vel: Vec2::ZERO,
    ///     color: Color32::WHITE,
    ///     elapsed: 0.0,
    ///     lifetime: 0.5,
    /// };
    /// assert_eq!(p.life_ratio(), 1.0);
    /// ```
    pub fn life_ratio(&self) -> f32 {
        if self.lifetime <= 0.0 {
            0.0
        } else {
            (1.0 - (self.elapsed / self.lifetime)).clamp(0.0, 1.0)
        }
    }
}

/// 削除文字パーティクル群の物理更新・描画マネージャ。
///
/// 疑似乱数（Xorshift64）を内包し、外部クレートへ依存せずに自然な散乱初速を付与します。
///
/// # Examples
/// ```
/// use egui::{Color32, Pos2};
/// use vivivi_gui::ParticleManager;
///
/// let mut mgr = ParticleManager::new();
/// mgr.spawn_char('R', Pos2::new(100.0, 100.0), Color32::RED);
/// assert!(mgr.is_active());
/// for _ in 0..12 {
///     mgr.update(0.05);
/// }
/// assert!(!mgr.is_active());
/// ```
#[derive(Debug, Clone)]
pub struct ParticleManager {
    /// アクティブなパーティクル配列
    pub particles: Vec<Particle>,
    /// 重力加速度 (px/s²)
    pub gravity: f32,
    /// エフェクト有効フラグ
    pub enabled: bool,
    /// 軽量 Xorshift64 疑似乱数シード
    rng_state: u64,
}

impl Default for ParticleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ParticleManager {
    /// デフォルトの重力加速度（`720.0 px/s²`）および有効状態で新しい [`ParticleManager`] を初期化します。
    ///
    /// # Examples
    /// ```
    /// use vivivi_gui::ParticleManager;
    ///
    /// let mgr = ParticleManager::new();
    /// assert!(mgr.enabled);
    /// assert!(!mgr.is_active());
    /// ```
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(64),
            gravity: DEFAULT_GRAVITY,
            enabled: true,
            rng_state: 0x853c_49e6_748f_ea9b,
        }
    }

    /// 現在アクティブなパーティクルが存在するかどうかを返します。
    ///
    /// このメソッドが `true` の間のみ `ctx.request_repaint()` を要求し、
    /// `false` の場合は再描画要求を行わず省電力リアクティブモードを維持します。
    ///
    /// # Examples
    /// ```
    /// use egui::{Color32, Pos2};
    /// use vivivi_gui::ParticleManager;
    ///
    /// let mut mgr = ParticleManager::new();
    /// assert!(!mgr.is_active());
    /// mgr.spawn_char('x', Pos2::ZERO, Color32::WHITE);
    /// assert!(mgr.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        self.enabled && !self.particles.is_empty()
    }

    /// 現在保持しているパーティクルの数を返します。
    pub fn len(&self) -> usize {
        self.particles.len()
    }

    /// パーティクルが空かどうかを返します。
    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    /// 保持しているすべてのパーティクルを消去します。
    pub fn clear(&mut self) {
        self.particles.clear();
    }

    /// 0.0 〜 1.0 の疑似乱数を生成します（Xorshift64）。
    fn next_f32(&mut self) -> f32 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        ((x >> 40) as u32 as f32) / ((1u32 << 24) as f32)
    }

    /// 指定された文字と初期座標から落下パーティクルを1つ生成・追加します。
    ///
    /// 空白文字（スペース・タブ・改行）は無視されます。同時パーティクル数が上限（256個）に達している場合は、
    /// 最も古いパーティクルを破棄してフレームレートを維持します。
    ///
    /// # Examples
    /// ```
    /// use egui::{Color32, Pos2};
    /// use vivivi_gui::ParticleManager;
    ///
    /// let mut mgr = ParticleManager::new();
    /// mgr.spawn_char(' ', Pos2::ZERO, Color32::WHITE);
    /// assert_eq!(mgr.particles.len(), 0);
    /// mgr.spawn_char('V', Pos2::new(50.0, 80.0), Color32::RED);
    /// assert_eq!(mgr.particles.len(), 1);
    /// ```
    pub fn spawn_char(&mut self, ch: char, pos: Pos2, color: Color32) {
        if !self.enabled || ch.is_whitespace() {
            return;
        }

        // 上限超過時は最も古いパーティクルを破棄して追加
        if self.particles.len() >= MAX_PARTICLES {
            self.particles.remove(0);
        }

        // 初速: 左右へランダムに散らばりつつ、少し上へ跳ね上がってから落下する
        let vx = (self.next_f32() * 2.0 - 1.0) * 85.0;
        let vy = -50.0 - self.next_f32() * 90.0;

        self.particles.push(Particle {
            ch,
            pos,
            vel: Vec2::new(vx, vy),
            color,
            elapsed: 0.0,
            lifetime: DEFAULT_LIFETIME,
        });
    }

    /// 経過時間 `dt`（秒）に基づいて全パーティクルの位置・速度を更新し、
    /// 寿命（0.5秒）を超えたパーティクルを消滅させます。
    ///
    /// 極端なフレームドロップ時にも座標計算が発散しないよう、`dt` は内部で `0.001..=0.05` 秒にクランプされます。
    ///
    /// # Examples
    /// ```
    /// use egui::{Color32, Pos2};
    /// use vivivi_gui::ParticleManager;
    ///
    /// let mut mgr = ParticleManager::new();
    /// mgr.spawn_char('a', Pos2::new(10.0, 10.0), Color32::WHITE);
    /// for _ in 0..15 {
    ///     mgr.update(0.04);
    /// }
    /// assert!(!mgr.is_active());
    /// ```
    pub fn update(&mut self, dt: f32) {
        if self.particles.is_empty() {
            return;
        }

        // 極端なフレームスパイク時も物理挙動が破綻しないよう dt をクランプ
        let step_dt = dt.clamp(0.001, 0.05);

        for p in &mut self.particles {
            p.elapsed += step_dt;
            p.vel.y += self.gravity * step_dt;
            p.pos += p.vel * step_dt;
        }

        self.particles.retain(|p| p.elapsed < p.lifetime);
    }

    /// アクティブな全パーティクルを [`Painter`] 上にフェードアウト付きで描画します。
    pub fn render(&self, painter: &Painter, font_id: &FontId) {
        if !self.enabled || self.particles.is_empty() {
            return;
        }

        for p in &self.particles {
            let ratio = p.life_ratio();
            let alpha = (ratio * 255.0).round() as u8;
            if alpha == 0 {
                continue;
            }

            let faded_color =
                Color32::from_rgba_unmultiplied(p.color.r(), p.color.g(), p.color.b(), alpha);

            painter.text(
                p.pos,
                Align2::LEFT_TOP,
                p.ch.encode_utf8(&mut [0u8; 4]),
                font_id.clone(),
                faded_color,
            );
        }
    }
}

/// 削除前後の行文字列を比較し、削除された文字スライス（開始文字インデックスと削除文字ベクタ）を抽出します。
///
/// 文字数が増えていた場合や同一文字列の場合は `None` を返します。
///
/// # Examples
/// ```
/// use vivivi_gui::extract_deleted_line_segment;
///
/// let (start, chars) = extract_deleted_line_segment("hello", "hllo").unwrap();
/// assert_eq!(start, 1);
/// assert_eq!(chars, vec!['e']);
/// ```
pub fn extract_deleted_line_segment(before: &str, after: &str) -> Option<(usize, Vec<char>)> {
    let b_chars: Vec<char> = before.chars().collect();
    let a_chars: Vec<char> = after.chars().collect();

    if b_chars.len() <= a_chars.len() {
        return None;
    }

    let mut prefix = 0;
    while prefix < a_chars.len() && b_chars[prefix] == a_chars[prefix] {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix < (a_chars.len() - prefix)
        && b_chars[b_chars.len() - 1 - suffix] == a_chars[a_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let end = b_chars.len().saturating_sub(suffix);
    if end > prefix {
        Some((prefix, b_chars[prefix..end].to_vec()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_max_cap_enforced() {
        let mut mgr = ParticleManager::new();
        for _ in 0..(MAX_PARTICLES + 20) {
            mgr.spawn_char('X', Pos2::new(10.0, 10.0), Color32::WHITE);
        }
        assert_eq!(mgr.particles.len(), MAX_PARTICLES);
    }
}
