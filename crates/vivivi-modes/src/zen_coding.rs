//! Zen-Coding (Emmet) 展開モジュール。
//!
//! ViVi 4 の Zen-Coding 機能（`Ctrl-,`）に準拠し、
//! HTML/XML の短縮省略記法（例: `ul.menu>li*3>a`）を解析して正規の構造タグへ展開します。

/// 自立閉じタグ（Void Elements）一覧
const VOID_TAGS: &[&str] = &["br", "hr", "img", "input", "meta", "link"];

/// Zen-Coding の構文木ノード。
///
/// タグ名、ID、クラス名リスト、繰り返し回数、子要素リストを保持します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::zen_coding::ZenElement;
///
/// let el = ZenElement::new("div".to_string());
/// assert_eq!(el.tag, "div");
/// assert_eq!(el.count, 1);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ZenElement {
    /// タグ名（例: `"div"`, `"span"`, `"li"`）
    pub tag: String,
    /// ID 属性（`#` で指定）
    pub id: Option<String>,
    /// クラス名リスト（`.` で指定）
    pub classes: Vec<String>,
    /// 繰り返し展開数（`*N` で指定）
    pub count: usize,
    /// 子要素リスト（`>` で指定）
    pub children: Vec<ZenElement>,
}

impl ZenElement {
    /// デフォルト設定で指定タグ名の `ZenElement` を生成します。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::zen_coding::ZenElement;
    ///
    /// let el = ZenElement::new("p".to_string());
    /// assert_eq!(el.tag, "p");
    /// ```
    pub fn new(tag: String) -> Self {
        Self {
            tag,
            id: None,
            classes: Vec::new(),
            count: 1,
            children: Vec::new(),
        }
    }

    /// ノードおよび子要素を HTML 文字列に再帰的にレンダリングします。
    ///
    /// # Parameters
    /// - `indent_level`: インデントの深さ。
    /// - `indent_str`: 1階層あたりのインデント文字列（例: `"  "`）。
    ///
    /// # Examples
    ///
    /// ```
    /// use vivivi_modes::zen_coding::ZenElement;
    ///
    /// let el = ZenElement::new("span".to_string());
    /// let html = el.render(0, "  ");
    /// assert_eq!(html.trim(), "<span></span>");
    /// ```
    pub fn render(&self, indent_level: usize, indent_str: &str) -> String {
        let mut rendered = String::new();

        for _ in 0..self.count {
            let indent = indent_str.repeat(indent_level);
            let mut open_tag = format!("<{}", self.tag);

            if let Some(ref id) = self.id {
                open_tag.push_str(&format!(" id=\"{id}\""));
            }

            if !self.classes.is_empty() {
                open_tag.push_str(&format!(" class=\"{}\"", self.classes.join(" ")));
            }

            if self.tag == "a" {
                open_tag.push_str(" href=\"\"");
            } else if self.tag == "img" {
                open_tag.push_str(" src=\"\" alt=\"\"");
            } else if self.tag == "input" {
                open_tag.push_str(" type=\"text\"");
            }

            if VOID_TAGS.contains(&self.tag.as_str()) {
                open_tag.push_str(" />");
                rendered.push_str(&format!("{indent}{open_tag}\n"));
            } else if self.children.is_empty() {
                open_tag.push_str(&format!("></{}>", self.tag));
                rendered.push_str(&format!("{indent}{open_tag}\n"));
            } else {
                open_tag.push('>');
                rendered.push_str(&format!("{indent}{open_tag}\n"));

                for child in &self.children {
                    rendered.push_str(&child.render(indent_level + 1, indent_str));
                }

                rendered.push_str(&format!("{indent}</{}>\n", self.tag));
            }
        }

        rendered
    }
}

/// Zen-Coding 短縮文字列をパースして正規の HTML 文字列へ展開します。
///
/// ID記号（`#`）、クラス記号（`.`）、子要素階層（`>`）、同階層要素（`+`）、
/// 繰り返し（`*N`）に対応します。入力が空または構文不正の場合は `None` を返します。
///
/// # Examples
///
/// ```
/// use vivivi_modes::zen_coding::expand_zen_coding;
///
/// let html = expand_zen_coding("div#main.container").unwrap();
/// assert_eq!(html, "<div id=\"main\" class=\"container\"></div>");
///
/// let list = expand_zen_coding("ul>li*2").unwrap();
/// assert_eq!(list, "<ul>\n  <li></li>\n  <li></li>\n</ul>");
/// ```
pub fn expand_zen_coding(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 兄弟要素 (+) でトップレベルを分割
    let siblings = split_by_top_level(trimmed, '+');
    let mut root_elements = Vec::new();

    for sib in siblings {
        if let Some(el) = parse_chain(sib) {
            root_elements.push(el);
        } else {
            return None;
        }
    }

    let mut result = String::new();
    for el in root_elements {
        result.push_str(&el.render(0, "  "));
    }

    Some(result.trim_end().to_string())
}

/// 子要素チェーン (>) をパース
fn parse_chain(input: &str) -> Option<ZenElement> {
    let parts = split_by_top_level(input, '>');
    if parts.is_empty() {
        return None;
    }

    let mut root = parse_single_element(parts[0])?;
    let mut current = &mut root;

    for part in &parts[1..] {
        let child = parse_single_element(part)?;
        current.children.push(child);
        current = current.children.last_mut()?;
    }

    Some(root)
}

/// 単一要素 (例: `div#main.foo.bar*3`) をパース
fn parse_single_element(input: &str) -> Option<ZenElement> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 乗算 (*N) の分離
    let (base, count) = if let Some(idx) = trimmed.find('*') {
        let cnt = trimmed[idx + 1..].trim().parse::<usize>().unwrap_or(1);
        (&trimmed[..idx], cnt)
    } else {
        (trimmed, 1)
    };

    #[derive(PartialEq)]
    enum Mode {
        Tag,
        Id,
        Class,
    }

    let mut tag = String::new();
    let mut id = None;
    let mut classes = Vec::new();

    let mut current_token = String::new();
    let mut mode = Mode::Tag;

    for ch in base.chars() {
        match ch {
            '#' => {
                match mode {
                    Mode::Tag => tag = current_token.clone(),
                    Mode::Id => id = Some(current_token.clone()),
                    Mode::Class => classes.push(current_token.clone()),
                }
                current_token.clear();
                mode = Mode::Id;
            }
            '.' => {
                match mode {
                    Mode::Tag => tag = current_token.clone(),
                    Mode::Id => id = Some(current_token.clone()),
                    Mode::Class => classes.push(current_token.clone()),
                }
                current_token.clear();
                mode = Mode::Class;
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    match mode {
        Mode::Tag => tag = current_token,
        Mode::Id => id = Some(current_token),
        Mode::Class => classes.push(current_token),
    }

    let tag_name = if tag.is_empty() {
        "div".to_string()
    } else {
        tag
    };

    let mut el = ZenElement::new(tag_name);
    el.id = id;
    el.classes = classes;
    el.count = count;
    Some(el)
}

/// トップレベルの特定区切り文字で分割
fn split_by_top_level(input: &str, delim: char) -> Vec<&str> {
    input
        .split(delim)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_tag() {
        assert_eq!(expand_zen_coding("div").unwrap(), "<div></div>");
        assert_eq!(expand_zen_coding("br").unwrap(), "<br />");
        assert_eq!(expand_zen_coding("a").unwrap(), "<a href=\"\"></a>");
    }

    #[test]
    fn test_id_and_classes() {
        assert_eq!(
            expand_zen_coding("div#main.container.fluid").unwrap(),
            "<div id=\"main\" class=\"container fluid\"></div>"
        );
    }

    #[test]
    fn test_children_and_multiplication() {
        let expected = "<ul>\n  <li></li>\n  <li></li>\n  <li></li>\n</ul>";
        assert_eq!(expand_zen_coding("ul>li*3").unwrap(), expected);
    }

    #[test]
    fn test_siblings() {
        let expected = "<h1></h1>\n<p></p>";
        assert_eq!(expand_zen_coding("h1+p").unwrap(), expected);
    }
}
