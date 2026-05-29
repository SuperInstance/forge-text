# forge-text

Text decomposition into tiles for Plato agents.

Text is the most common input format. This crate decomposes documents, logs, articles, and chat messages into typed `TextTile` units for downstream processing.

## Types

### TextTile

```rust
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
```

### TextKind

```rust
pub enum TextKind {
    Paragraph, Sentence, Line, Word, Token,
    Heading, ListItem, CodeBlock, Blockquote,
    Custom(String),
}
```

### TextStats

```rust
pub struct TextStats {
    pub total_chars: usize,
    pub total_words: usize,
    pub avg_tile_size: f64,
    pub tile_count: usize,
}
```

## Usage

```rust
use forge_text::TextDecomposer;

let d = TextDecomposer::new();

// Decompose by paragraphs (markdown-aware)
let tiles = d.decompose_paragraphs("# Heading\n\nSome text.\n\n- list item");

// Decompose by sentences, lines, or words
let sentences = d.decompose_sentences("Hello. World!");
let lines = d.decompose_lines("line one\nline two");
let words = d.decompose_words("hello world");

// Detect language (simple heuristic)
let lang = d.detect_language("This is English text."); // Some("en")

// Reconstruct text from tiles
let text = d.reconstruct(&tiles);

// Get stats
let stats = d.stats(&tiles);
```

## Markdown Awareness

The decomposer detects and tags markdown structures:

- **Headings** — lines starting with `#`
- **List items** — lines starting with `- `, `* `, `+ `
- **Code blocks** — fenced with ` ``` `
- **Blockquotes** — lines starting with `> `

## Dependencies

- `serde` + `serde_json` — serialization
- `uuid` — unique tile IDs

## License

MIT
