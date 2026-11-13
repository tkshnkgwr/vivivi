//! プロジェクト可視化グラフ（Visualize / 力学モデルグラフ）モジュール。
//!
//! ViVi 4 で好評を博したプロジェクトファイル群の力学モデル（Force-Directed Graph）を
//! Rust / egui 上で再現します。
//! ファイルサイズ（対数スケール半径）、更新日時の鮮度（カラー発光）、
//! および拡張子別の色分けを行い、ノードクリックで即座に対象ファイルを開きます。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use egui::{Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use crate::theme::colors;

/// グラフ上の1ノード（ファイルまたはディレクトリ）
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// 一意なノードID
    pub id: usize,
    /// ファイルまたはディレクトリのパス
    pub path: PathBuf,
    /// 表示名
    pub name: String,
    /// ディレクトリであるか
    pub is_dir: bool,
    /// ファイルサイズ（バイト）
    pub size_bytes: u64,
    /// 最終更新日時
    pub modified: SystemTime,
    /// 2D 物理座標（グラフ空間）
    pub pos: Pos2,
    /// 物理速度ベクトル
    pub vel: Vec2,
    /// 加わる累積合力ベクトル
    pub force: Vec2,
    /// ノードの描画半径 (px)
    pub radius: f32,
    /// 更新鮮度に基づく背景カラー
    pub fill_color: Color32,
    /// 拡張子に応じた枠線カラー
    pub stroke_color: Color32,
    /// マウスドラッグ等によるピン留め固定中か
    pub is_pinned: bool,
}

/// ノード間の接続関係（親子階層または関連依存）
#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// 接続元ノードID
    pub from: usize,
    /// 接続先ノードID
    pub to: usize,
    /// バネの自然長 (px)
    pub rest_length: f32,
    /// エッジの色
    pub color: Color32,
}

/// プロジェクト可視化グラフ管理構造体
pub struct ForceGraph {
    /// グラフノード一覧
    pub nodes: Vec<GraphNode>,
    /// グラフエッジ一覧
    pub edges: Vec<GraphEdge>,
    /// パンオフセット（平行移動ベクトル）
    pub pan: Vec2,
    /// ズーム倍率（1.0 = 等倍）
    pub zoom: f32,
    /// 現在ドラッグ中のノードID
    pub dragged_node: Option<usize>,
    /// 選択/ホバー中のノードID
    pub hovered_node: Option<usize>,
    /// 物理演算が収束して静止状態（スリープ）にあるか
    pub is_sleeping: bool,
}

impl ForceGraph {
    /// 新規空グラフを作成します。
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            pan: Vec2::ZERO,
            zoom: 1.0,
            dragged_node: None,
            hovered_node: None,
            is_sleeping: false,
        }
    }

    /// 指定ディレクトリをスキャンして力学グラフを構築します（最大 100 項目）。
    pub fn build_from_dir(root_dir: &Path) -> Self {
        let mut graph = Self::new();
        let mut node_map: HashMap<PathBuf, usize> = HashMap::new();

        // ルートディレクトリ自身をノードとして追加
        let root_name = root_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("root")
            .to_string();

        let root_id = 0;
        graph.nodes.push(GraphNode {
            id: root_id,
            path: root_dir.to_path_buf(),
            name: root_name,
            is_dir: true,
            size_bytes: 0,
            modified: SystemTime::now(),
            pos: Pos2::ZERO,
            vel: Vec2::ZERO,
            force: Vec2::ZERO,
            radius: 26.0,
            fill_color: colors::ACCENT_PURPLE,
            stroke_color: colors::ACCENT_CYAN,
            is_pinned: false,
        });
        node_map.insert(root_dir.to_path_buf(), root_id);

        let mut queue = vec![root_dir.to_path_buf()];
        let max_nodes = 80;

        let mut angle: f32 = 0.0;
        let now = SystemTime::now();

        while let Some(current_dir) = queue.pop() {
            if graph.nodes.len() >= max_nodes {
                break;
            }

            let entries = match std::fs::read_dir(&current_dir) {
                Ok(rd) => rd.filter_map(|e| e.ok()).collect::<Vec<_>>(),
                Err(_) => continue,
            };

            let parent_id = *node_map.get(&current_dir).unwrap_or(&root_id);

            for entry in entries {
                if graph.nodes.len() >= max_nodes {
                    break;
                }

                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                // 巨大なバイナリ・キャッシュディレクトリ・隠しファイルは除外
                if file_name.starts_with('.')
                    || file_name == "target"
                    || file_name == "node_modules"
                    || file_name == ".git"
                {
                    continue;
                }

                let meta = match std::fs::symlink_metadata(&path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let is_dir = meta.is_dir();
                let size_bytes = if is_dir { 0 } else { meta.len() };
                let modified = meta.modified().unwrap_or(now);

                // ノード半径（ファイルサイズの対数スケール）
                let radius = if is_dir {
                    20.0
                } else {
                    let kb = (size_bytes as f32) / 1024.0;
                    (14.0 + 3.0 * (1.0 + kb).ln()).clamp(14.0, 36.0)
                };

                // カラー設定
                let fill_color = Self::calc_freshness_color(now, modified, is_dir);
                let stroke_color = Self::calc_extension_color(&path, is_dir);

                // 円周上に初期配置
                angle += 0.45;
                let dist = 120.0 + (graph.nodes.len() as f32) * 5.0;
                let init_pos = Pos2::new(dist * angle.cos(), dist * angle.sin());

                let id = graph.nodes.len();
                graph.nodes.push(GraphNode {
                    id,
                    path: path.clone(),
                    name: file_name,
                    is_dir,
                    size_bytes,
                    modified,
                    pos: init_pos,
                    vel: Vec2::ZERO,
                    force: Vec2::ZERO,
                    radius,
                    fill_color,
                    stroke_color,
                    is_pinned: false,
                });
                node_map.insert(path.clone(), id);

                // 親ノードとエッジで結ぶ
                graph.edges.push(GraphEdge {
                    from: parent_id,
                    to: id,
                    rest_length: 80.0,
                    color: Color32::from_rgba_unmultiplied(86, 95, 137, 120),
                });

                if is_dir {
                    queue.push(path);
                }
            }
        }

        graph
    }

    /// 更新日時の鮮度に応じた発光カラーを算出します。
    fn calc_freshness_color(now: SystemTime, modified: SystemTime, is_dir: bool) -> Color32 {
        if is_dir {
            return Color32::from_rgb(41, 46, 66); // 落ち着いたフォルダ色
        }

        let elapsed = now
            .duration_since(modified)
            .unwrap_or(Duration::from_secs(86400 * 30));

        if elapsed < Duration::from_secs(3600) {
            // 直近 1時間以内: 鮮烈なシアン（更新直後）
            Color32::from_rgb(125, 207, 255)
        } else if elapsed < Duration::from_secs(86400) {
            // 24時間以内: ゴールド
            Color32::from_rgb(224, 175, 104)
        } else if elapsed < Duration::from_secs(86400 * 7) {
            // 1週間以内: ブルー
            Color32::from_rgb(122, 162, 247)
        } else {
            // それ以前: ディミングダーク
            Color32::from_rgb(36, 40, 59)
        }
    }

    /// 拡張子に応じた境界線カラーを算出します。
    fn calc_extension_color(path: &Path, is_dir: bool) -> Color32 {
        if is_dir {
            return colors::ACCENT_PURPLE;
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        match ext.as_str() {
            "rs" => Color32::from_rgb(255, 158, 100), // Rust Orange
            "md" | "txt" => Color32::from_rgb(158, 206, 106), // Markdown Green
            "toml" | "json" | "yaml" | "yml" => Color32::from_rgb(187, 154, 247), // Config Purple
            "js" | "ts" | "jsx" | "tsx" => Color32::from_rgb(224, 175, 104), // Web Yellow
            "c" | "cpp" | "h" | "hpp" => Color32::from_rgb(122, 162, 247), // C/C++ Blue
            _ => Color32::from_rgb(120, 124, 153),    // その他 Gray
        }
    }

    /// 力学モデルの物理ステップを実行します。
    ///
    /// 物理運動がアクティブであれば `true` を返し、収束して静止していれば `false` を返します。
    pub fn update_physics(&mut self, dt: f32) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        // 1. 各ノードの合力をリセット
        for node in &mut self.nodes {
            node.force = Vec2::ZERO;
        }

        let node_count = self.nodes.len();

        // 2. クーロン反発力 (全ノードペア間 F = k / d^2)
        const K_REPULSION: f32 = 7000.0;
        for i in 0..node_count {
            for j in (i + 1)..node_count {
                let delta = self.nodes[i].pos - self.nodes[j].pos;
                let dist_sq = delta.length_sq().max(25.0); // ゼロ除算防止
                let dist = dist_sq.sqrt();
                let force_mag = K_REPULSION / dist_sq;
                let force_vec = (delta / dist) * force_mag;

                self.nodes[i].force += force_vec;
                self.nodes[j].force -= force_vec;
            }
        }

        // 3. バネ引力 (エッジ間 F = k * (d - rest_length))
        const K_SPRING: f32 = 0.06;
        for edge in &self.edges {
            if edge.from >= node_count || edge.to >= node_count {
                continue;
            }
            let delta = self.nodes[edge.to].pos - self.nodes[edge.from].pos;
            let dist = delta.length().max(1.0);
            let displacement = dist - edge.rest_length;
            let force_mag = K_SPRING * displacement;
            let force_vec = (delta / dist) * force_mag;

            self.nodes[edge.from].force += force_vec;
            self.nodes[edge.to].force -= force_vec;
        }

        // 4. 中心引き戻し力 (Central Gravity)
        const K_CENTER: f32 = 0.006;
        for node in &mut self.nodes {
            let center_force = -node.pos.to_vec2() * K_CENTER;
            node.force += center_force;
        }

        // 5. 速度更新・摩擦減衰 (Damping)・位置積分
        const DAMPING: f32 = 0.85;
        let mut total_kinetic_energy: f32 = 0.0;

        for node in &mut self.nodes {
            if node.is_pinned {
                node.vel = Vec2::ZERO;
                continue;
            }

            node.vel = (node.vel + node.force * dt) * DAMPING;
            // 最大速度を制限（暴走防止）
            let speed = node.vel.length();
            if speed > 180.0 {
                node.vel = (node.vel / speed) * 180.0;
            }

            node.pos += node.vel * dt;
            total_kinetic_energy += node.vel.length_sq();
        }

        // 省電力ハイブリッド制御: 運動エネルギーが極小ならスリープへ移行
        let is_moving = total_kinetic_energy > 0.05;
        self.is_sleeping = !is_moving;

        is_moving
    }

    /// グラフ描画およびユーザーインタラクション（ドラッグ、クリックで開く、パン、ズーム）
    ///
    /// ユーザーがファイルノードをクリックした場合、そのファイルの `PathBuf` を返します。
    pub fn ui(&mut self, ui: &mut egui::Ui, view_rect: Rect) -> Option<PathBuf> {
        let mut file_to_open: Option<PathBuf> = None;
        let dt = ui.input(|i| i.stable_dt).clamp(0.001, 0.05);

        // 物理シミュレーションを更新
        let is_moving = self.update_physics(dt);
        if is_moving || self.dragged_node.is_some() {
            ui.ctx().request_repaint();
        }

        let center = view_rect.center();
        let painter = ui.painter_at(view_rect);

        // 背景描画（アクティブテーマの背景色を使用）
        painter.rect_filled(view_rect, 0.0, ui.visuals().window_fill);

        // パン & ズームの入力受付
        let response = ui.allocate_rect(view_rect, Sense::click_and_drag());

        // ホイールによるズーム
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0
            && view_rect.contains(ui.input(|i| i.pointer.hover_pos().unwrap_or(Pos2::ZERO)))
        {
            let zoom_factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
            self.zoom = (self.zoom * zoom_factor).clamp(0.2, 3.5);
            self.is_sleeping = false;
        }

        // 右ボタンドラッグまたは中ボタンドラッグによるパン
        if response.dragged_by(egui::PointerButton::Secondary)
            || response.dragged_by(egui::PointerButton::Middle)
        {
            self.pan += response.drag_delta();
            self.is_sleeping = false;
        }

        let to_screen = |pos: Pos2| -> Pos2 { center + (pos.to_vec2() * self.zoom) + self.pan };

        let to_graph = |screen_pos: Pos2| -> Pos2 {
            Pos2::new(
                (screen_pos.x - center.x - self.pan.x) / self.zoom,
                (screen_pos.y - center.y - self.pan.y) / self.zoom,
            )
        };

        // 1. エッジの描画
        for edge in &self.edges {
            if edge.from < self.nodes.len() && edge.to < self.nodes.len() {
                let p1 = to_screen(self.nodes[edge.from].pos);
                let p2 = to_screen(self.nodes[edge.to].pos);
                painter.line_segment([p1, p2], Stroke::new(1.2 * self.zoom, edge.color));
            }
        }

        // 2. マウスポインター位置の判定
        let mouse_pos = ui.input(|i| i.pointer.hover_pos());
        self.hovered_node = None;

        if let Some(mpos) = mouse_pos {
            for node in self.nodes.iter().rev() {
                let screen_node_pos = to_screen(node.pos);
                let screen_radius = node.radius * self.zoom;
                if mpos.distance(screen_node_pos) <= screen_radius {
                    self.hovered_node = Some(node.id);
                    break;
                }
            }
        }

        // 3. ノードのドラッグ処理
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(hovered) = self.hovered_node
        {
            self.dragged_node = Some(hovered);
            self.nodes[hovered].is_pinned = true;
            self.is_sleeping = false;
        }

        if response.dragged_by(egui::PointerButton::Primary)
            && let Some(dragged) = self.dragged_node
            && let Some(mpos) = mouse_pos
        {
            self.nodes[dragged].pos = to_graph(mpos);
            self.nodes[dragged].vel = Vec2::ZERO;
            self.is_sleeping = false;
        }

        if response.drag_stopped_by(egui::PointerButton::Primary)
            && let Some(dragged) = self.dragged_node
        {
            self.nodes[dragged].is_pinned = false;
            self.dragged_node = None;
        }

        // 4. クリック判定（ファイルオープン）
        if response.clicked_by(egui::PointerButton::Primary)
            && let Some(hovered) = self.hovered_node
        {
            let node = &self.nodes[hovered];
            if !node.is_dir {
                file_to_open = Some(node.path.clone());
            }
        }

        // 5. ノードの描画
        for node in &self.nodes {
            let screen_pos = to_screen(node.pos);
            let screen_radius = node.radius * self.zoom;
            let is_hovered = self.hovered_node == Some(node.id);

            // ホバー時の発光外枠
            if is_hovered {
                painter.circle_stroke(
                    screen_pos,
                    screen_radius + 4.0 * self.zoom,
                    Stroke::new(2.5 * self.zoom, colors::ACCENT_CYAN),
                );
            }

            // ノード塗りつぶし
            painter.circle_filled(screen_pos, screen_radius, node.fill_color);

            // ノード外枠
            let stroke_width = if is_hovered { 2.2 } else { 1.4 } * self.zoom;
            painter.circle_stroke(
                screen_pos,
                screen_radius,
                Stroke::new(stroke_width, node.stroke_color),
            );

            // ノード名テキスト描画
            let font_size = (11.0 * self.zoom).clamp(8.0, 16.0);
            let text_color = if is_hovered {
                ui.visuals().strong_text_color()
            } else {
                ui.visuals().text_color()
            };

            let text_pos = screen_pos + Vec2::new(0.0, screen_radius + 3.0 * self.zoom);
            painter.text(
                text_pos,
                egui::Align2::CENTER_TOP,
                &node.name,
                FontId::proportional(font_size),
                text_color,
            );
        }

        // 6. 操作ガイドバッジを上部にオーバーレイ描画
        let guide_rect = Rect::from_min_size(
            view_rect.min + Vec2::new(12.0, 12.0),
            Vec2::new(view_rect.width() - 24.0, 28.0),
        );
        let badge_bg = ui.visuals().panel_fill.gamma_multiply(0.9);
        painter.rect_filled(guide_rect, 4.0, badge_bg);
        painter.text(
            guide_rect.left_center() + Vec2::new(10.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "💡 左ドラッグ: ノード移動 | クリック: ファイルを開く | 右ドラッグ: パン | ホイール: ズーム | Esc: エディタへ戻る",
            FontId::proportional(12.0),
            ui.visuals().weak_text_color(),
        );

        file_to_open
    }
}

impl Default for ForceGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_graph_build_and_step() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut graph = ForceGraph::build_from_dir(&manifest_dir);

        assert!(!graph.nodes.is_empty());
        // 物理ステップを1回実行
        let moving = graph.update_physics(0.016);
        // 初期状態では反発力と引力により運動が発生する
        assert!(moving);
        assert!(!graph.is_sleeping);
    }

    #[test]
    fn test_freshness_and_extension_colors() {
        let now = SystemTime::now();
        let c_recent = ForceGraph::calc_freshness_color(now, now, false);
        assert_eq!(c_recent, Color32::from_rgb(125, 207, 255));

        let c_rs = ForceGraph::calc_extension_color(Path::new("src/main.rs"), false);
        assert_eq!(c_rs, Color32::from_rgb(255, 158, 100));
    }
}
