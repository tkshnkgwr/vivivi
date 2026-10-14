#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Markdown Table Vertical Aligner
East Asian Width (全角文字=幅2) を考慮して、Markdownテーブルのパイプとハイフンを完全に垂直整列します。
"""

import sys
import unicodedata
from pathlib import Path


def char_width(c: str) -> int:
    w = unicodedata.east_asian_width(c)
    return 2 if w in ("F", "W") else 1


def str_width(s: str) -> int:
    return sum(char_width(c) for c in s)


def pad_cell(text: str, target_width: int, align: str) -> str:
    curr_w = str_width(text)
    diff = max(0, target_width - curr_w)
    if align == "center":
        left = diff // 2
        right = diff - left
        return " " * left + text + " " * right
    elif align == "right":
        return " " * diff + text
    else:  # left
        return text + " " * diff


def align_table_block(table_lines: list[str]) -> list[str]:
    rows = []
    for line in table_lines:
        trimmed = line.strip()
        if trimmed.startswith("|") and trimmed.endswith("|"):
            parts = [p.strip() for p in trimmed[1:-1].split("|")]
            rows.append(parts)
        else:
            rows.append([line])

    if len(rows) < 2:
        return table_lines

    col_count = max(len(r) for r in rows)
    # パディングして列数を統一
    for r in rows:
        while len(r) < col_count:
            r.append("")

    # 各列のアライメント検出 (行インデックス 1 がセパレータ行)
    aligns = []
    for cell in rows[1]:
        c = cell.strip()
        if c.startswith(":") and c.endswith(":"):
            aligns.append("center")
        elif c.endswith(":"):
            aligns.append("right")
        elif c.startswith(":"):
            aligns.append("left")
        else:
            aligns.append("left")

    # 列幅の最大値を計算（セパレータ行を除く）
    max_widths = [3] * col_count
    for i, r in enumerate(rows):
        if i == 1:
            continue
        for c_idx, cell in enumerate(r):
            w = str_width(cell)
            if w > max_widths[c_idx]:
                max_widths[c_idx] = w

    # 結果の構築
    result = []
    for i, r in enumerate(rows):
        if i == 1:
            # セパレータ行
            sep_cells = []
            for c_idx in range(col_count):
                align = aligns[c_idx]
                w = max_widths[c_idx]
                if align == "center":
                    sep = ":" + "-" * max(1, w - 2) + ":"
                elif align == "right":
                    sep = "-" * max(1, w - 1) + ":"
                elif align == "left":
                    sep = ":" + "-" * max(1, w - 1)
                else:
                    sep = "-" * w
                sep_cells.append(sep)
            result.append("| " + " | ".join(sep_cells) + " |")
        else:
            row_cells = []
            for c_idx in range(col_count):
                cell = r[c_idx]
                align = aligns[c_idx]
                row_cells.append(pad_cell(cell, max_widths[c_idx], align))
            result.append("| " + " | ".join(row_cells) + " |")

    return result


def process_file(filepath: Path) -> bool:
    content = filepath.read_text(encoding="utf-8")
    lines = content.splitlines()

    new_lines = []
    in_table = False
    table_lines = []

    for line in lines:
        stripped = line.strip()
        if stripped.startswith("|") and stripped.endswith("|"):
            in_table = True
            table_lines.append(line)
        else:
            if in_table:
                new_lines.extend(align_table_block(table_lines))
                table_lines = []
                in_table = False
            new_lines.append(line)

    if in_table:
        new_lines.extend(align_table_block(table_lines))

    new_content = "\n".join(new_lines) + "\n"
    if new_content != content:
        filepath.write_text(new_content, encoding="utf-8")
        return True
    return False


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python align_table.py <markdown_file_or_dir> ...")
        sys.exit(1)

    for arg in sys.argv[1:]:
        p = Path(arg)
        if p.is_file():
            changed = process_file(p)
            print(f"{'[UPDATED]' if changed else '[NO-OP]'} {p}")
        elif p.is_dir():
            for md in p.rglob("*.md"):
                changed = process_file(md)
                print(f"{'[UPDATED]' if changed else '[NO-OP]'} {md}")
