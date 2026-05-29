use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextTile {
    pub id: Uuid,
    pub kind: TextKind,
    pub content: String,
    pub index: u64,
    pub char_count: usize,
    pub word_count: usize,
    pub lang: Option<String>,
    pub meta: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TextKind {
    Paragraph,
    Sentence,
    Line,
    Word,
    Token,
    Heading,
    ListItem,
    CodeBlock,
    Blockquote,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStats {
    pub total_chars: usize,
    pub total_words: usize,
    pub avg_tile_size: f64,
    pub tile_count: usize,
}

pub struct TextDecomposer;

impl TextDecomposer {
    pub fn new() -> Self {
        TextDecomposer
    }

    /// Decompose text into paragraphs (separated by blank lines).
    pub fn decompose_paragraphs(&self, text: &str) -> Vec<TextTile> {
        let mut tiles = Vec::new();
        let mut index: u64 = 0;
        let mut current = String::new();
        let mut in_code = false;

        for line in text.lines() {
            let trimmed = line.trim();

            // Track fenced code blocks
            if trimmed.starts_with("```") {
                if in_code {
                    // End of code block — emit it as one tile
                    current.push_str(line);
                    current.push('\n');
                    in_code = false;
                    let content = current.trim().to_string();
                    if !content.is_empty() {
                        tiles.push(self.make_tile(content, TextKind::CodeBlock, index, &HashMap::new()));
                        index += 1;
                    }
                    current.clear();
                    continue;
                } else {
                    // Flush any accumulated paragraph before starting code block
                    if !current.trim().is_empty() {
                        let content = current.trim().to_string();
                        let kind = self.detect_block_kind(&content);
                        tiles.push(self.make_tile(content, kind, index, &HashMap::new()));
                        index += 1;
                        current.clear();
                    }
                    in_code = true;
                    current.push_str(line);
                    current.push('\n');
                    continue;
                }
            }

            if in_code {
                current.push_str(line);
                current.push('\n');
                continue;
            }

            if trimmed.is_empty() {
                // Blank line = paragraph boundary
                if !current.trim().is_empty() {
                    let content = current.trim().to_string();
                    let kind = self.detect_block_kind(&content);
                    tiles.push(self.make_tile(content, kind, index, &HashMap::new()));
                    index += 1;
                    current.clear();
                }
            } else {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(trimmed);
            }
        }

        // Flush remaining
        if !current.trim().is_empty() {
            let content = current.trim().to_string();
            let kind = if in_code { TextKind::CodeBlock } else { self.detect_block_kind(&content) };
            tiles.push(self.make_tile(content, kind, index, &HashMap::new()));
        }

        tiles
    }

    /// Decompose text into sentences.
    pub fn decompose_sentences(&self, text: &str) -> Vec<TextTile> {
        let mut tiles = Vec::new();
        let mut index: u64 = 0;
        let mut current = String::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            current.push(ch);

            if ch == '.' || ch == '!' || ch == '?' {
                // Check if this is really a sentence boundary
                // Not an abbreviation (simple heuristic: next char is space or end, and current word is not short uppercase)
                let next_is_space_or_end = i + 1 >= chars.len() || chars[i + 1].is_whitespace();
                let prev_word_short = {
                    let w: String = current.chars().rev().skip(1).take_while(|c| c.is_alphanumeric()).collect::<Vec<_>>().into_iter().rev().collect();
                    w.len() <= 2 && w.chars().all(|c| c.is_uppercase())
                };

                if next_is_space_or_end && !prev_word_short {
                    let content = current.trim().to_string();
                    if !content.is_empty() {
                        tiles.push(self.make_tile(content, TextKind::Sentence, index, &HashMap::new()));
                        index += 1;
                    }
                    current.clear();
                }
            }

            i += 1;
        }

        if !current.trim().is_empty() {
            tiles.push(self.make_tile(current.trim().to_string(), TextKind::Sentence, index, &HashMap::new()));
        }

        tiles
    }

    /// Decompose text into lines.
    pub fn decompose_lines(&self, text: &str) -> Vec<TextTile> {
        text.lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(i, line)| {
                let trimmed = line.trim();
                let kind = self.detect_line_kind(trimmed);
                self.make_tile(trimmed.to_string(), kind, i as u64, &HashMap::new())
            })
            .collect()
    }

    /// Decompose text into words.
    pub fn decompose_words(&self, text: &str) -> Vec<TextTile> {
        text.split_whitespace()
            .enumerate()
            .map(|(i, word)| {
                self.make_tile(word.to_string(), TextKind::Word, i as u64, &HashMap::new())
            })
            .collect()
    }

    /// Simple language detection heuristic.
    pub fn detect_language(&self, text: &str) -> Option<String> {
        let lower = text.to_lowercase();
        let common_en = ["the ", "is ", "are ", "and ", "of ", "in ", "to ", "it ", "that ", "this "];
        let common_es = [" el ", " la ", " los ", " las ", " de ", " en ", " es ", " que ", " por ", " con "];
        let common_fr = [" le ", " la ", " les ", " de ", " des ", " en ", " est ", " que ", " un ", " une "];
        let common_de = [" der ", " die ", " das ", " und ", " ist ", " ein ", " eine ", " auf ", " mit ", " nicht "];

        let en_count = common_en.iter().filter(|w| lower.contains(*w)).count();
        let es_count = common_es.iter().filter(|w| lower.contains(*w)).count();
        let fr_count = common_fr.iter().filter(|w| lower.contains(*w)).count();
        let de_count = common_de.iter().filter(|w| lower.contains(*w)).count();

        let max = en_count.max(es_count).max(fr_count).max(de_count);
        if max == 0 {
            return None;
        }

        if en_count == max { return Some("en".to_string()); }
        if es_count == max { return Some("es".to_string()); }
        if fr_count == max { return Some("fr".to_string()); }
        if de_count == max { return Some("de".to_string()); }
        None
    }

    /// Reconstruct text from tiles (sorted by index).
    pub fn reconstruct(&self, tiles: &[TextTile]) -> String {
        let mut sorted: Vec<&TextTile> = tiles.iter().collect();
        sorted.sort_by_key(|t| t.index);

        // Choose separator based on kinds
        let has_paragraphs = sorted.iter().any(|t| t.kind == TextKind::Paragraph || t.kind == TextKind::CodeBlock || t.kind == TextKind::Heading);
        let sep = if has_paragraphs { "\n\n" } else { " " };

        sorted.iter().map(|t| t.content.as_str()).collect::<Vec<_>>().join(sep)
    }

    /// Compute stats over tiles.
    pub fn stats(&self, tiles: &[TextTile]) -> TextStats {
        let tile_count = tiles.len();
        let total_chars: usize = tiles.iter().map(|t| t.char_count).sum();
        let total_words: usize = tiles.iter().map(|t| t.word_count).sum();
        let avg_tile_size = if tile_count > 0 {
            total_chars as f64 / tile_count as f64
        } else {
            0.0
        };

        TextStats {
            total_chars,
            total_words,
            avg_tile_size,
            tile_count,
        }
    }

    // --- helpers ---

    fn make_tile(&self, content: String, kind: TextKind, index: u64, meta: &HashMap<String, String>) -> TextTile {
        let char_count = content.chars().count();
        let word_count = content.split_whitespace().count();
        TextTile {
            id: Uuid::new_v4(),
            kind,
            content,
            index,
            char_count,
            word_count,
            lang: None,
            meta: meta.clone(),
        }
    }

    fn detect_block_kind(&self, content: &str) -> TextKind {
        if content.starts_with('#') {
            TextKind::Heading
        } else if content.starts_with("> ") || content.starts_with('>') {
            TextKind::Blockquote
        } else if content.starts_with("- ") || content.starts_with("* ") || content.starts_with("+ ") {
            TextKind::ListItem
        } else {
            TextKind::Paragraph
        }
    }

    fn detect_line_kind(&self, line: &str) -> TextKind {
        if line.starts_with("```") {
            TextKind::CodeBlock
        } else if line.starts_with('#') {
            TextKind::Heading
        } else if line.starts_with("> ") || line.starts_with('>') {
            TextKind::Blockquote
        } else if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
            TextKind::ListItem
        } else if line.starts_with("```") {
            TextKind::CodeBlock
        } else {
            TextKind::Line
        }
    }
}

impl Default for TextDecomposer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decomposer() -> TextDecomposer {
        TextDecomposer::new()
    }

    #[test]
    fn test_decompose_paragraphs_basic() {
        let d = decomposer();
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird one.";
        let tiles = d.decompose_paragraphs(text);
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0].kind, TextKind::Paragraph);
        assert_eq!(tiles[0].content, "First paragraph.");
        assert_eq!(tiles[1].content, "Second paragraph.");
        assert_eq!(tiles[2].content, "Third one.");
    }

    #[test]
    fn test_decompose_paragraphs_heading() {
        let d = decomposer();
        let text = "# Title\n\nSome content under it.";
        let tiles = d.decompose_paragraphs(text);
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0].kind, TextKind::Heading);
        assert_eq!(tiles[0].content, "# Title");
    }

    #[test]
    fn test_decompose_paragraphs_list_item() {
        let d = decomposer();
        let text = "- Item one\n- Item two\n\nA paragraph.";
        let tiles = d.decompose_paragraphs(text);
        // Lines get joined into one block until blank line
        assert!(tiles.iter().any(|t| t.kind == TextKind::ListItem || t.kind == TextKind::Paragraph));
    }

    #[test]
    fn test_decompose_paragraphs_blockquote() {
        let d = decomposer();
        let text = "> This is a quote\n\nNormal text.";
        let tiles = d.decompose_paragraphs(text);
        assert_eq!(tiles[0].kind, TextKind::Blockquote);
    }

    #[test]
    fn test_decompose_paragraphs_code_block() {
        let d = decomposer();
        let text = "Some text\n\n```\nfn main() {}\n```\n\nMore text.";
        let tiles = d.decompose_paragraphs(text);
        assert!(tiles.iter().any(|t| t.kind == TextKind::CodeBlock));
        let code_tile = tiles.iter().find(|t| t.kind == TextKind::CodeBlock).unwrap();
        assert!(code_tile.content.contains("fn main()"));
    }

    #[test]
    fn test_decompose_sentences() {
        let d = decomposer();
        let text = "Hello world. How are you? I am fine!";
        let tiles = d.decompose_sentences(text);
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0].content, "Hello world.");
        assert_eq!(tiles[1].content, "How are you?");
        assert_eq!(tiles[2].content, "I am fine!");
    }

    #[test]
    fn test_decompose_lines() {
        let d = decomposer();
        let text = "line one\nline two\nline three";
        let tiles = d.decompose_lines(text);
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0].content, "line one");
        assert!(tiles.iter().all(|t| t.kind == TextKind::Line));
    }

    #[test]
    fn test_decompose_lines_markdown() {
        let d = decomposer();
        let text = "# Heading\n> quote\n- list item\nnormal\n```code```";
        let tiles = d.decompose_lines(text);
        assert_eq!(tiles[0].kind, TextKind::Heading);
        assert_eq!(tiles[1].kind, TextKind::Blockquote);
        assert_eq!(tiles[2].kind, TextKind::ListItem);
        assert_eq!(tiles[3].kind, TextKind::Line);
        assert_eq!(tiles[4].kind, TextKind::CodeBlock);
    }

    #[test]
    fn test_decompose_words() {
        let d = decomposer();
        let text = "hello world foo";
        let tiles = d.decompose_words(text);
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0].content, "hello");
        assert_eq!(tiles[1].content, "world");
        assert_eq!(tiles[2].content, "foo");
        assert!(tiles.iter().all(|t| t.kind == TextKind::Word));
    }

    #[test]
    fn test_detect_language_english() {
        let d = decomposer();
        assert_eq!(d.detect_language("This is the best thing in the world and it is great."), Some("en".to_string()));
    }

    #[test]
    fn test_detect_language_none() {
        let d = decomposer();
        assert_eq!(d.detect_language("xyz"), None);
    }

    #[test]
    fn test_reconstruct_paragraphs() {
        let d = decomposer();
        let tiles = d.decompose_paragraphs("Hello world.\n\nGoodbye world.");
        let result = d.reconstruct(&tiles);
        assert_eq!(result, "Hello world.\n\nGoodbye world.");
    }

    #[test]
    fn test_reconstruct_words() {
        let d = decomposer();
        let tiles = d.decompose_words("a b c");
        let result = d.reconstruct(&tiles);
        assert_eq!(result, "a b c");
    }

    #[test]
    fn test_stats() {
        let d = decomposer();
        let tiles = d.decompose_words("hello world foo bar");
        let s = d.stats(&tiles);
        assert_eq!(s.tile_count, 4);
        assert_eq!(s.total_words, 4);
        assert_eq!(s.total_chars, 5 + 5 + 3 + 3); // hello(5) world(5) foo(3) bar(3) = 16
        assert!((s.avg_tile_size - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_empty_input() {
        let d = decomposer();
        assert!(d.decompose_paragraphs("").is_empty());
        assert!(d.decompose_sentences("").is_empty());
        assert!(d.decompose_lines("").is_empty());
        assert!(d.decompose_words("").is_empty());
        let s = d.stats(&[]);
        assert_eq!(s.tile_count, 0);
    }

    #[test]
    fn test_unicode() {
        let d = decomposer();
        let tiles = d.decompose_words("こんにちは 世界");
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0].char_count, 5); // こんにちは
        assert_eq!(tiles[1].char_count, 2); // 世界
    }

    #[test]
    fn test_long_text() {
        let d = decomposer();
        let text: String = (0..1000).map(|i| format!("Word{}", i)).collect::<Vec<_>>().join(" ");
        let tiles = d.decompose_words(&text);
        assert_eq!(tiles.len(), 1000);
        let s = d.stats(&tiles);
        assert_eq!(s.tile_count, 1000);
    }

    #[test]
    fn test_tile_has_uuid() {
        let d = decomposer();
        let tiles = d.decompose_words("test");
        assert_eq!(tiles.len(), 1);
        assert!(!tiles[0].id.to_string().is_empty());
    }

    #[test]
    fn test_detect_language_spanish() {
        let d = decomposer();
        let text = "Esta es la historia de los días que vienen con el mundo.";
        assert_eq!(d.detect_language(text), Some("es".to_string()));
    }
}
