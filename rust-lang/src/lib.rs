#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Keyword,
    KeywordType,
    String,
    Comment,
    Number,
    Function,
    Macro,
    Normal,
}

impl TokenKind {
    /// Returns the color category name for the IDE theme
    pub fn color_category(&self) -> &'static str {
        match self {
            TokenKind::Keyword => "keyword",   // purple #c586c0
            TokenKind::KeywordType => "type",  // blue #569cd6
            TokenKind::String => "string",     // orange #ce9178
            TokenKind::Comment => "comment",   // green #6a9955
            TokenKind::Number => "number",     // light green #b5cea8
            TokenKind::Function => "function", // yellow #dcdcaa
            TokenKind::Macro => "macro",       // yellow #dcdcaa
            TokenKind::Normal => "normal",     // gray #d4d4d4
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub kind: TokenKind,
}

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "box", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "unsafe", "use", "where", "while",
];

const TYPE_KEYWORDS: &[&str] = &[
    "bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
    "f32", "f64", "char", "str", "String", "Vec", "Option", "Result", "Box", "Arc", "Rc",
];

fn char_byte_offset(chars: &[char], char_idx: usize) -> usize {
    chars[..char_idx].iter().map(|c| c.len_utf8()).sum()
}

/// Tokenize a single line of Rust source code.
pub fn tokenize_line(line: &str) -> Vec<Token> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return vec![Token {
            text: line.to_string(),
            kind: TokenKind::Comment,
        }];
    }

    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let offset = char_byte_offset(&chars, i);

        // Line comment
        if line[offset..].starts_with("//") {
            tokens.push(Token {
                text: line[offset..].to_string(),
                kind: TokenKind::Comment,
            });
            break;
        }

        // String or char literal
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            let mut s = String::new();
            s.push(quote);
            i += 1;
            while i < len {
                let c = chars[i];
                s.push(c);
                i += 1;
                if c == '\\' && i < len {
                    s.push(chars[i]);
                    i += 1;
                } else if c == quote {
                    break;
                }
            }
            tokens.push(Token {
                text: s,
                kind: TokenKind::String,
            });
            continue;
        }

        // Number
        if chars[i].is_ascii_digit() {
            let mut s = String::new();
            while i < len
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_')
            {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(Token {
                text: s,
                kind: TokenKind::Number,
            });
            continue;
        }

        // Word (identifier, keyword, type, macro, function)
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            // Macro: word followed by '!'
            let kind = if i < len && chars[i] == '!' {
                TokenKind::Macro
            } else if TYPE_KEYWORDS.contains(&word.as_str()) {
                TokenKind::KeywordType
            } else if KEYWORDS.contains(&word.as_str()) {
                TokenKind::Keyword
            } else if i < len && chars[i] == '(' {
                TokenKind::Function
            } else {
                TokenKind::Normal
            };
            tokens.push(Token { text: word, kind });
            continue;
        }

        // Non-word punctuation: collect until next meaningful boundary
        let start = i;
        while i < len
            && !chars[i].is_alphabetic()
            && chars[i] != '_'
            && !chars[i].is_ascii_digit()
            && chars[i] != '"'
            && chars[i] != '\''
            && !line[char_byte_offset(&chars, i)..].starts_with("//")
        {
            i += 1;
        }
        if i > start {
            let punct: String = chars[start..i].iter().collect();
            if !punct.is_empty() {
                tokens.push(Token {
                    text: punct,
                    kind: TokenKind::Normal,
                });
            }
        }
    }

    tokens
}

// ── FFI interface ────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn language_id() -> *const std::ffi::c_char {
    c"rust".as_ptr()
}

#[no_mangle]
pub extern "C" fn file_extensions() -> *const std::ffi::c_char {
    c"rs".as_ptr()
}

/// Tokenize a line of Rust code. Returns JSON: [{"text":"...","kind":"keyword"}, ...]
///
/// # Safety
/// `line_ptr` must be a valid, nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn tokenize_line_ffi(
    line_ptr: *const std::ffi::c_char,
) -> *mut std::ffi::c_char {
    let line = unsafe { std::ffi::CStr::from_ptr(line_ptr).to_str().unwrap_or("") };
    let tokens = tokenize_line(line);
    let json = tokens_to_json(&tokens);
    let c_str = std::ffi::CString::new(json).unwrap();
    c_str.into_raw()
}

/// # Safety
/// `ptr` must have been returned by `tokenize_line_ffi`.
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() {
        unsafe { drop(std::ffi::CString::from_raw(ptr)) }
    }
}

/// Scan Rust source text for a definition of `word` and return a code-fenced signature.
/// Returns `None` if nothing is found.
pub fn hover_info(word: &str, file_content: &str) -> Option<String> {
    let fn_patterns = [
        format!("fn {}(", word),
        format!("fn {} (", word),
        format!("pub fn {}(", word),
        format!("pub(crate) fn {}(", word),
        format!("async fn {}(", word),
        format!("pub async fn {}(", word),
        format!("unsafe fn {}(", word),
        format!("pub unsafe fn {}(", word),
    ];
    let type_patterns = [
        format!("struct {} ", word),
        format!("struct {}{}", word, '{'),
        format!("pub struct {}", word),
        format!("enum {} ", word),
        format!("pub enum {}", word),
        format!("trait {} ", word),
        format!("pub trait {}", word),
        format!("type {} =", word),
        format!("pub type {} =", word),
    ];
    let let_patterns = [
        format!("let {}: ", word),
        format!("let mut {}: ", word),
        format!("let {} =", word),
        format!("let mut {} =", word),
        format!("const {}: ", word),
        format!("static {}: ", word),
    ];

    for line in file_content.lines() {
        let trimmed = line.trim();
        for pat in &fn_patterns {
            if trimmed.contains(pat.as_str()) {
                let sig = trimmed.trim_end_matches('{').trim_end();
                return Some(format!("```rust\n{sig}\n```"));
            }
        }
        for pat in &type_patterns {
            if trimmed.contains(pat.as_str()) || trimmed.starts_with(pat.as_str()) {
                let sig = trimmed.trim_end_matches('{').trim_end();
                return Some(format!("```rust\n{sig}\n```"));
            }
        }
        for pat in &let_patterns {
            if trimmed.starts_with(pat.as_str()) {
                let end = trimmed
                    .find('=')
                    .or_else(|| trimmed.find(';'))
                    .unwrap_or(trimmed.len());
                let sig = trimmed[..end].trim_end();
                return Some(format!("```rust\n{sig}\n```"));
            }
        }
    }
    None
}

/// FFI wrapper: look up hover info for `word` in `file_content_ptr`.
/// Returns a heap-allocated C string, or null if not found.
/// Caller must free the result with `free_string`.
///
/// # Safety
/// Both pointer arguments must be valid nul-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn hover_info_ffi(
    word_ptr: *const std::ffi::c_char,
    file_content_ptr: *const std::ffi::c_char,
) -> *mut std::ffi::c_char {
    let word = unsafe { std::ffi::CStr::from_ptr(word_ptr).to_str().unwrap_or("") };
    let content = unsafe { std::ffi::CStr::from_ptr(file_content_ptr).to_str().unwrap_or("") };
    match hover_info(word, content) {
        Some(s) => std::ffi::CString::new(s).map(|c| c.into_raw()).unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

fn tokens_to_json(tokens: &[Token]) -> String {
    let parts: Vec<String> = tokens
        .iter()
        .map(|t| {
            format!(
                r#"{{"text":{},"kind":"{}"}}"#,
                serde_json_escape(&t.text),
                t.kind.color_category()
            )
        })
        .collect();
    format!("[{}]", parts.join(","))
}

fn serde_json_escape(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{}\"", escaped)
}

/// Returns the LSP server command for Rust: rust-analyzer
pub fn lsp_server_command() -> Option<(String, Vec<String>)> {
    Some(("rust-analyzer".to_string(), vec![]))
}

// ── README ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword() {
        let tokens = tokenize_line("fn main() {");
        assert!(tokens
            .iter()
            .any(|t| t.text == "fn" && t.kind == TokenKind::Keyword));
        assert!(tokens
            .iter()
            .any(|t| t.text == "main" && t.kind == TokenKind::Function));
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize_line("// this is a comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_string() {
        let tokens = tokenize_line(r#"let s = "hello";"#);
        assert!(tokens
            .iter()
            .any(|t| t.text == "\"hello\"" && t.kind == TokenKind::String));
    }

    #[test]
    fn test_number() {
        let tokens = tokenize_line("let x = 42;");
        assert!(tokens
            .iter()
            .any(|t| t.text == "42" && t.kind == TokenKind::Number));
    }

    #[test]
    fn test_macro() {
        let tokens = tokenize_line("println!(\"hello\");");
        assert!(tokens
            .iter()
            .any(|t| t.text.starts_with("println") && t.kind == TokenKind::Macro));
    }

    #[test]
    fn test_type_keyword() {
        let tokens = tokenize_line("let x: Vec<String> = Vec::new();");
        assert!(tokens
            .iter()
            .any(|t| t.text == "Vec" && t.kind == TokenKind::KeywordType));
    }

    #[test]
    fn test_doc_comment() {
        let tokens = tokenize_line("/// This is a doc comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_char_literal() {
        let tokens = tokenize_line("let c = 'a';");
        assert!(tokens
            .iter()
            .any(|t| t.text == "'a'" && t.kind == TokenKind::String));
    }

    #[test]
    fn test_let_mut_keyword() {
        let tokens = tokenize_line("let mut x = 0;");
        assert!(tokens
            .iter()
            .any(|t| t.text == "let" && t.kind == TokenKind::Keyword));
        assert!(tokens
            .iter()
            .any(|t| t.text == "mut" && t.kind == TokenKind::Keyword));
    }
}
