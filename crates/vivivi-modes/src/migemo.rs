//! ローマ字日本語検索 (Migemo / `g/` コマンド) モジュール。
//!
//! ViVi 3 / ViVi 4 の「ローマ字検索 (`migemo.html`, `vivi_vi.html`)」に準拠し、
//! IMEをONにすることなく、アルファベット（ローマ字）入力によって
//! ひらがな、カタカナ、およびSKK辞書による漢字候補を横断検索する正規表現パターンを構築します。

use regex::Regex;
use std::collections::HashMap;

/// ローマ字からひらがな・カタカナへの変換テーブル要素。
struct RomajiEntry {
    romaji: &'static str,
    hiragana: &'static str,
    katakana: &'static str,
}

macro_rules! romaji_map {
    ( $( ($r:expr, $h:expr, $k:expr) ),* $(,)? ) => {
        &[
            $(
                RomajiEntry {
                    romaji: $r,
                    hiragana: $h,
                    katakana: $k,
                },
            )*
        ]
    };
}

/// ローマ字表記からひらがな・カタカナへの対応テーブル。
/// 3文字以上の拗音を優先的にマッチさせるため、文字数降順でソートして適用します。
static ROMAJI_TABLE: &[RomajiEntry] = romaji_map![
    // 3文字（拗音・長音・促音等）
    ("kya", "きゃ", "キャ"),
    ("kyu", "きゅ", "キュ"),
    ("kyo", "きょ", "キョ"),
    ("sha", "しゃ", "シャ"),
    ("shu", "しゅ", "シュ"),
    ("sho", "しょ", "ショ"),
    ("she", "しぇ", "シェ"),
    ("cha", "ちゃ", "チャ"),
    ("chu", "ちゅ", "チュ"),
    ("cho", "ちょ", "チョ"),
    ("che", "ちぇ", "チェ"),
    ("nya", "にゃ", "ニャ"),
    ("nyu", "にゅ", "ニュ"),
    ("nyo", "にょ", "ニョ"),
    ("hya", "ひゃ", "ヒャ"),
    ("hyu", "ひゅ", "ヒュ"),
    ("hyo", "ひょ", "ヒョ"),
    ("mya", "みゃ", "ミャ"),
    ("myu", "みゅ", "ミュ"),
    ("myo", "みょ", "ミョ"),
    ("rya", "りゃ", "リャ"),
    ("ryu", "りゅ", "リュ"),
    ("ryo", "りょ", "リョ"),
    ("gya", "ぎゃ", "ギャ"),
    ("gyu", "ぎゅ", "ギュ"),
    ("gyo", "ぎょ", "ギョ"),
    ("ja", "じゃ", "ジャ"),
    ("ju", "じゅ", "ジュ"),
    ("jo", "じょ", "ジョ"),
    ("jya", "じゃ", "ジャ"),
    ("jyu", "じゅ", "ジュ"),
    ("jyo", "じょ", "ジョ"),
    ("bya", "びゃ", "ビャ"),
    ("byu", "びゅ", "ビュ"),
    ("byo", "びょ", "ビョ"),
    ("pya", "ぴゃ", "ピャ"),
    ("pyu", "ぴゅ", "ピュ"),
    ("pyo", "ぴょ", "ピョ"),
    ("tsu", "つ", "ツ"),
    // 2文字
    ("ka", "か", "カ"),
    ("ki", "き", "キ"),
    ("ku", "く", "ク"),
    ("ke", "け", "ケ"),
    ("ko", "こ", "コ"),
    ("sa", "さ", "サ"),
    ("si", "し", "シ"),
    ("shi", "し", "シ"),
    ("su", "す", "ス"),
    ("se", "せ", "セ"),
    ("so", "そ", "ソ"),
    ("ta", "た", "タ"),
    ("ti", "ち", "チ"),
    ("chi", "ち", "チ"),
    ("tu", "つ", "ツ"),
    ("te", "て", "テ"),
    ("to", "と", "ト"),
    ("na", "な", "ナ"),
    ("ni", "に", "ニ"),
    ("nu", "ぬ", "ヌ"),
    ("ne", "ね", "ネ"),
    ("no", "の", "ノ"),
    ("ha", "は", "ハ"),
    ("hi", "ひ", "ヒ"),
    ("fu", "ふ", "フ"),
    ("hu", "ふ", "フ"),
    ("he", "へ", "ヘ"),
    ("ho", "ほ", "ホ"),
    ("ma", "ま", "マ"),
    ("mi", "み", "ミ"),
    ("mu", "む", "ム"),
    ("me", "め", "メ"),
    ("mo", "も", "モ"),
    ("ya", "や", "ヤ"),
    ("yu", "ゆ", "ユ"),
    ("yo", "よ", "ヨ"),
    ("ra", "ら", "ラ"),
    ("ri", "り", "リ"),
    ("ru", "る", "ル"),
    ("re", "れ", "レ"),
    ("ro", "ろ", "ロ"),
    ("wa", "わ", "ワ"),
    ("wo", "を", "ヲ"),
    ("ga", "が", "ガ"),
    ("gi", "ぎ", "ギ"),
    ("gu", "ぐ", "グ"),
    ("ge", "げ", "ゲ"),
    ("go", "ご", "ゴ"),
    ("za", "ざ", "ザ"),
    ("zi", "じ", "ジ"),
    ("ji", "じ", "ジ"),
    ("zu", "ず", "ズ"),
    ("ze", "ぜ", "ゼ"),
    ("zo", "ぞ", "ゾ"),
    ("da", "だ", "ダ"),
    ("di", "ぢ", "ヂ"),
    ("du", "づ", "ヅ"),
    ("de", "で", "デ"),
    ("do", "ど", "ド"),
    ("ba", "ば", "バ"),
    ("bi", "び", "ビ"),
    ("bu", "ぶ", "ブ"),
    ("be", "べ", "ベ"),
    ("bo", "ぼ", "ボ"),
    ("pa", "ぱ", "パ"),
    ("pi", "ぴ", "ピ"),
    ("pu", "ぷ", "プ"),
    ("pe", "ぺ", "ペ"),
    ("po", "ぽ", "ポ"),
    ("nn", "ん", "ン"),
    // 1文字（母音）
    ("a", "あ", "ア"),
    ("i", "い", "イ"),
    ("u", "う", "ウ"),
    ("e", "え", "エ"),
    ("o", "お", "オ"),
];

/// ローマ字日本語検索（Migemo）エンジン。
///
/// ローマ字入力からひらがな、カタカナ、漢字を網羅した正規表現パターンを構築し、
/// IME不要のインクリメンタル日本語検索を実現します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::migemo::MigemoEngine;
///
/// let mut migemo = MigemoEngine::new();
/// migemo.add_dict_entry("にほん", "日本");
/// let re = migemo.compile_query("nihon").unwrap();
/// assert!(re.is_match("日本"));
/// assert!(re.is_match("にほん"));
/// assert!(re.is_match("ニホン"));
/// assert!(re.is_match("nihon"));
/// ```
/// 代表的な頻出日本語単語の静的組み込みSKK辞書エントリ（読み -> 漢字表記候補リスト）。
static BUILTIN_SKK_ENTRIES: &[(&str, &[&str])] = &[
    ("けんさく", &["検索", "見索"]),
    ("にほん", &["日本", "二本"]),
    ("とうきょう", &["東京"]),
    ("せかい", &["世界"]),
    ("せってい", &["設定"]),
    ("ふぁいる", &["ファイル"]),
    ("ほぞん", &["保存"]),
    ("しゅうりょう", &["終了"]),
    ("さくじょ", &["削除"]),
    ("いどう", &["移動"]),
    ("かいし", &["開始"]),
    ("ぎょう", &["行"]),
    ("もじ", &["文字"]),
    ("がめん", &["画面"]),
    ("へんしゅう", &["編集"]),
    ("かんすう", &["関数"]),
    ("へんすう", &["変数"]),
    ("かた", &["型", "方"]),
    ("てすと", &["テスト"]),
    ("かくにん", &["確認"]),
    ("ぶんかつ", &["分割"]),
    ("さくせい", &["作成"]),
    ("こうしん", &["更新"]),
    ("しゅつりょく", &["出力"]),
    ("にゅうりょく", &["入力"]),
    ("ちかん", &["置換"]),
    ("せつぞく", &["接続"]),
    ("じっこう", &["実行"]),
    ("けっか", &["結果"]),
    ("せいこう", &["成功"]),
    ("しっぱい", &["失敗"]),
    ("じょうたい", &["状態"]),
    ("げんざい", &["現在"]),
    ("してい", &["指定"]),
    ("せんたく", &["選択"]),
    ("さんしょう", &["参照"]),
    ("じょうほう", &["情報"]),
    ("りれき", &["履歴"]),
    ("へんこう", &["変更"]),
    ("ついか", &["追加"]),
    ("ひょうじ", &["表示"]),
    ("ひひょうじ", &["非表示"]),
    ("けいせん", &["罫線"]),
    ("ほご", &["保護"]),
    ("はいらいと", &["ハイライト"]),
    ("もーど", &["モード"]),
    ("こまんど", &["コマンド"]),
    ("うぃんどう", &["ウィンドウ"]),
    ("ばっふぁ", &["バッファ"]),
    ("かーそる", &["カーソル"]),
    ("すくろーる", &["スクロール"]),
    ("じしょ", &["辞書"]),
    ("ことば", &["言葉"]),
    ("かいけつ", &["解決"]),
    ("もんだい", &["問題"]),
    ("きょう", &["今日"]),
    ("あす", &["明日"]),
    ("あした", &["明日"]),
    ("きのう", &["昨日", "機能"]),
    ("ほんじつ", &["本日"]),
    ("かいはつ", &["開発"]),
    ("じっそう", &["実装"]),
    ("こーど", &["コード"]),
    ("そーす", &["ソース"]),
    ("ぷろぐらむ", &["プログラム"]),
    ("えでぃた", &["エディタ"]),
    ("だいけんじゃ", &["大賢者"]),
    ("ぼす", &["ボス"]),
];

/// ローマ字日本語検索（Migemo）エンジン。
///
/// ローマ字音節テーブルおよび組み込みSKK辞書を保持し、
/// 入力されたローマ字クエリから正規表現パターンを構築します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::migemo::MigemoEngine;
///
/// let engine = MigemoEngine::new();
/// let re = engine.compile_query("nihon").unwrap();
/// assert!(re.is_match("日本"));
/// assert!(re.is_match("にほん"));
/// assert!(re.is_match("ニホン"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct MigemoEngine {
    /// SKK辞書やカスタム単語辞書（読み（ひらがな） -> 漢字候補リスト）
    dict: HashMap<String, Vec<String>>,
}

impl MigemoEngine {
    /// 内蔵基本辞書を標準展開した `MigemoEngine` を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::migemo::MigemoEngine;
    ///
    /// let engine = MigemoEngine::new();
    /// let pattern = engine.query_to_regex_pattern("kensaku");
    /// assert!(pattern.contains("検索"));
    /// ```
    pub fn new() -> Self {
        let mut engine = Self::default();
        for &(yomi, candidates) in BUILTIN_SKK_ENTRIES {
            let entry = engine.dict.entry(yomi.to_string()).or_default();
            for &c in candidates {
                if !entry.contains(&c.to_string()) {
                    entry.push(c.to_string());
                }
            }
        }
        engine
    }

    /// 辞書に単語（ひらがな読み -> 漢字表記）を直接登録します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::migemo::MigemoEngine;
    ///
    /// let mut engine = MigemoEngine::new();
    /// engine.add_dict_entry("けんさく", "検索");
    /// let pattern = engine.query_to_regex_pattern("kensaku");
    /// assert!(pattern.contains("検索"));
    /// ```
    pub fn add_dict_entry(&mut self, yomi: &str, kanji: &str) {
        self.dict
            .entry(yomi.to_string())
            .or_default()
            .push(kanji.to_string());
    }

    /// SKK辞書形式のテキスト（またはファイル内容）をパースして登録します。
    ///
    /// 書式: `見出し /候補1/候補2;注釈/`
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::migemo::MigemoEngine;
    ///
    /// let mut engine = MigemoEngine::new();
    /// engine.load_skk_dict_str("とうきょう /東京/とうきょう;ひらがな/\n");
    /// let pattern = engine.query_to_regex_pattern("toukyou");
    /// assert!(pattern.contains("東京"));
    /// ```
    pub fn load_skk_dict_str(&mut self, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with(";;") || line.is_empty() {
                continue;
            }
            if let Some((yomi, candidates_str)) = line.split_once(' ') {
                let yomi = yomi.trim();
                let candidates: Vec<String> = candidates_str
                    .split('/')
                    .filter_map(|c| {
                        let c = c.trim();
                        if c.is_empty() {
                            None
                        } else {
                            // 注釈 ";..." を除去
                            Some(c.split(';').next().unwrap_or(c).to_string())
                        }
                    })
                    .collect();

                if !candidates.is_empty() {
                    let entry = self.dict.entry(yomi.to_string()).or_default();
                    for c in candidates {
                        if !entry.contains(&c) {
                            entry.push(c);
                        }
                    }
                }
            }
        }
    }

    /// ローマ字クエリ（例: `"nihon"`, `"kensaku"`）から、ひらがな・カタカナ・漢字を統合した
    /// 検索用正規表現パターン文字列を構築します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::migemo::MigemoEngine;
    ///
    /// let engine = MigemoEngine::new();
    /// let pattern = engine.query_to_regex_pattern("ka");
    /// assert!(pattern.contains("か"));
    /// assert!(pattern.contains("カ"));
    /// ```
    pub fn query_to_regex_pattern(&self, romaji: &str) -> String {
        let romaji_lower = romaji.to_ascii_lowercase();
        if romaji_lower.is_empty() {
            return String::new();
        }

        // 1. ローマ字をひらがなに変換
        let (hiragana, syllables) = self.parse_romaji(&romaji_lower);

        // 2. 辞書から完全一致または前方一致する漢字候補を探索
        let mut kanji_candidates = Vec::new();
        if let Some(candidates) = self.dict.get(&hiragana) {
            for c in candidates {
                kanji_candidates.push(regex::escape(c));
            }
        }

        // 3. 各音節ごとの (ひらがな|カタカナ|ローマ字) パターンを連結
        let mut syllable_pattern = String::new();
        for (r, h, k) in &syllables {
            syllable_pattern.push_str(&format!("(?:{}|{}|{})", h, k, regex::escape(r)));
        }

        // 4. 漢字候補があれば (漢字1|漢字2|音節パターン) として統合
        if kanji_candidates.is_empty() {
            syllable_pattern
        } else {
            format!("(?:{}|{})", kanji_candidates.join("|"), syllable_pattern)
        }
    }

    /// ローマ字クエリからコンパイル済みの正規表現（[`Regex`]）を構築します。
    ///
    /// # Errors
    /// 生成された正規表現パターンのコンパイルに失敗した場合に [`regex::Error`] を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::migemo::MigemoEngine;
    ///
    /// let engine = MigemoEngine::new();
    /// let re = engine.compile_query("sushi").unwrap();
    /// assert!(re.is_match("すし"));
    /// assert!(re.is_match("スシ"));
    /// ```
    pub fn compile_query(&self, romaji: &str) -> Result<Regex, regex::Error> {
        let pattern = self.query_to_regex_pattern(romaji);
        Regex::new(&pattern)
    }

    /// ローマ字文字列を音節リスト `(ローマ字, ひらがな, カタカナ)` にパースします。
    fn parse_romaji(&self, romaji: &str) -> (String, Vec<(String, String, String)>) {
        let mut syllables = Vec::new();
        let mut full_hiragana = String::new();
        let chars: Vec<char> = romaji.chars().collect();
        let mut idx = 0;

        while idx < chars.len() {
            // 促音の判定（子音の連続: 例 "kk", "tt", "ss"）
            if idx + 1 < chars.len()
                && chars[idx] == chars[idx + 1]
                && chars[idx].is_ascii_alphabetic()
                && !"aiueon".contains(chars[idx])
            {
                syllables.push((chars[idx].to_string(), "っ".to_string(), "ッ".to_string()));
                full_hiragana.push('っ');
                idx += 1;
                continue;
            }

            // テーブルから最長一致を探索
            let mut matched = false;
            for entry in ROMAJI_TABLE {
                let r_len = entry.romaji.len();
                if idx + r_len <= chars.len() {
                    let sub: String = chars[idx..idx + r_len].iter().collect();
                    if sub == entry.romaji {
                        syllables.push((
                            entry.romaji.to_string(),
                            entry.hiragana.to_string(),
                            entry.katakana.to_string(),
                        ));
                        full_hiragana.push_str(entry.hiragana);
                        idx += r_len;
                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                // 'n' の特別処理（単独の 'n' は "ん/ン/n" として扱う）
                if chars[idx] == 'n' {
                    syllables.push(("n".to_string(), "ん".to_string(), "ン".to_string()));
                    full_hiragana.push('ん');
                } else {
                    let ch_str = chars[idx].to_string();
                    syllables.push((ch_str.clone(), ch_str.clone(), ch_str));
                    full_hiragana.push(chars[idx]);
                }
                idx += 1;
            }
        }

        (full_hiragana, syllables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migemo_romaji_syllable_matching() {
        let migemo = MigemoEngine::new();
        let regex = migemo.compile_query("nihon").expect("should compile");

        // ひらがな
        assert!(regex.is_match("これはにほんです"));
        // カタカナ
        assert!(regex.is_match("これはニホンです"));
        // アルファベット
        assert!(regex.is_match("this is nihon"));
        // マッチしない
        assert!(!regex.is_match("これはとうきょうです"));
    }

    #[test]
    fn test_migemo_sokuon() {
        let migemo = MigemoEngine::new();
        let regex = migemo.compile_query("kettei").expect("should compile");

        assert!(regex.is_match("けってい"));
        assert!(regex.is_match("ケッテイ"));
        assert!(regex.is_match("kettei"));
    }

    #[test]
    fn test_migemo_with_skk_dict() {
        let mut migemo = MigemoEngine::new();
        // SKK辞書エントリを模倣して登録
        migemo.add_dict_entry("にほん", "日本");
        migemo.add_dict_entry("にほん", "二本");

        let regex = migemo.compile_query("nihon").expect("should compile");

        // 辞書による漢字マッチ
        assert!(regex.is_match("日本代表"));
        assert!(regex.is_match("二本のペン"));
        // ひらがな・カタカナも同時にマッチ
        assert!(regex.is_match("にほんご"));
        assert!(regex.is_match("ニホンカモシカ"));
    }

    #[test]
    fn test_load_skk_dict_string() {
        let mut migemo = MigemoEngine::new();
        // SKK辞書の見出し語はひらがな
        let skk_data = ";; SKK-JISYO sample\nけんさく /検索/見索/\nとうきょう /東京/\n";
        migemo.load_skk_dict_str(skk_data);

        let regex = migemo.compile_query("kensaku").expect("should compile");
        assert!(regex.is_match("全文検索エンジン"));
        assert!(regex.is_match("けんさく"));
        assert!(regex.is_match("ケンサク"));
        assert!(regex.is_match("kensaku"));
    }
}
