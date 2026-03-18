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
    pub fn color_category(&self) -> &'static str {
        match self {
            TokenKind::Keyword => "keyword",
            TokenKind::KeywordType => "type",
            TokenKind::String => "string",
            TokenKind::Comment => "comment",
            TokenKind::Number => "number",
            TokenKind::Function => "function",
            TokenKind::Macro => "macro",
            TokenKind::Normal => "normal",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub kind: TokenKind,
}

const KEYWORDS: &[&str] = &[
    "break", "case", "chan", "const", "continue", "default", "defer",
    "else", "fallthrough", "for", "func", "go", "goto", "if", "import",
    "interface", "map", "package", "range", "return", "select", "struct",
    "switch", "type", "var",
];

const TYPE_KEYWORDS: &[&str] = &[
    "bool", "byte", "complex64", "complex128", "error", "float32", "float64",
    "int", "int8", "int16", "int32", "int64", "rune", "string", "uint",
    "uint8", "uint16", "uint32", "uint64", "uintptr", "any", "comparable",
    "nil", "true", "false", "iota",
];

fn char_byte_offset(chars: &[char], char_idx: usize) -> usize {
    chars[..char_idx].iter().map(|c| c.len_utf8()).sum()
}

pub fn tokenize_line(line: &str) -> Vec<Token> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
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

        // String: " or ` (raw string)
        if chars[i] == '"' || chars[i] == '`' || chars[i] == '\'' {
            let quote = chars[i];
            let mut s = String::new();
            s.push(quote);
            i += 1;
            while i < len {
                let c = chars[i];
                s.push(c);
                i += 1;
                if c == '\\' && quote != '`' && i < len {
                    s.push(chars[i]);
                    i += 1;
                } else if c == quote {
                    break;
                }
            }
            tokens.push(Token { text: s, kind: TokenKind::String });
            continue;
        }

        // Number
        if chars[i].is_ascii_digit() {
            let mut s = String::new();
            while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_') {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(Token { text: s, kind: TokenKind::Number });
            continue;
        }

        // Word (identifier, keyword, type, function)
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let kind = if TYPE_KEYWORDS.contains(&word.as_str()) {
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

        // Punctuation
        let start = i;
        while i < len
            && !chars[i].is_alphabetic()
            && chars[i] != '_'
            && !chars[i].is_ascii_digit()
            && chars[i] != '"'
            && chars[i] != '\''
            && chars[i] != '`'
            && !line[char_byte_offset(&chars, i)..].starts_with("//")
        {
            i += 1;
        }
        if i > start {
            let punct: String = chars[start..i].iter().collect();
            if !punct.is_empty() {
                tokens.push(Token { text: punct, kind: TokenKind::Normal });
            }
        }
    }

    tokens
}

// ── FFI interface ─────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn language_id() -> *const std::ffi::c_char {
    c"go".as_ptr()
}

#[no_mangle]
pub extern "C" fn file_extensions() -> *const std::ffi::c_char {
    c"go".as_ptr()
}

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

pub fn hover_info(word: &str, file_content: &str) -> Option<String> {
    let fn_patterns = [
        format!("func {}(", word),
        format!("func {} (", word),
    ];
    let type_patterns = [
        format!("type {} struct", word),
        format!("type {} interface", word),
        format!("type {} =", word),
        format!("type {}\n", word),
    ];
    let var_patterns = [
        format!("var {}", word),
        format!("const {}", word),
        format!("{} :=", word),
        format!("{}: ", word),
    ];

    for line in file_content.lines() {
        let trimmed = line.trim();
        for pat in &fn_patterns {
            if trimmed.contains(pat.as_str()) {
                let sig = trimmed.trim_end_matches('{').trim_end();
                return Some(format!("```go\n{sig}\n```"));
            }
        }
        for pat in &type_patterns {
            if trimmed.contains(pat.as_str()) {
                let sig = trimmed.trim_end_matches('{').trim_end();
                return Some(format!("```go\n{sig}\n```"));
            }
        }
        for pat in &var_patterns {
            if trimmed.starts_with(pat.as_str()) {
                let end = trimmed.find('=').unwrap_or(trimmed.len());
                let sig = trimmed[..end].trim_end();
                return Some(format!("```go\n{sig}\n```"));
            }
        }
    }
    None
}

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

pub fn lsp_server_command() -> Option<(String, Vec<String>)> {
    Some(("gopls".to_string(), vec![]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword() {
        let tokens = tokenize_line("func main() {");
        assert!(tokens.iter().any(|t| t.text == "func" && t.kind == TokenKind::Keyword));
        assert!(tokens.iter().any(|t| t.text == "main" && t.kind == TokenKind::Function));
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize_line("// this is a comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_string() {
        let tokens = tokenize_line(r#"s := "hello""#);
        assert!(tokens.iter().any(|t| t.text == "\"hello\"" && t.kind == TokenKind::String));
    }

    #[test]
    fn test_raw_string() {
        let tokens = tokenize_line("s := `raw string`");
        assert!(tokens.iter().any(|t| t.text == "`raw string`" && t.kind == TokenKind::String));
    }

    #[test]
    fn test_number() {
        let tokens = tokenize_line("x := 42");
        assert!(tokens.iter().any(|t| t.text == "42" && t.kind == TokenKind::Number));
    }

    #[test]
    fn test_type_keyword() {
        let tokens = tokenize_line("var x int = 0");
        assert!(tokens.iter().any(|t| t.text == "int" && t.kind == TokenKind::KeywordType));
    }
}
