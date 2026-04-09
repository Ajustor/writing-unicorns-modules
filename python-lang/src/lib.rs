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
    "False", "None", "True", "and", "as", "assert", "async", "await",
    "break", "class", "continue", "def", "del", "elif", "else", "except",
    "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "while", "with", "yield",
];

const TYPE_KEYWORDS: &[&str] = &[
    "bool", "bytes", "bytearray", "complex", "dict", "float", "frozenset",
    "int", "list", "memoryview", "object", "range", "set", "str", "tuple", "type",
];

fn char_byte_offset(chars: &[char], char_idx: usize) -> usize {
    chars[..char_idx].iter().map(|c| c.len_utf8()).sum()
}

pub fn tokenize_line(line: &str) -> Vec<Token> {
    let trimmed = line.trim_start();

    // Full-line comment
    if trimmed.starts_with('#') {
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

        // Inline comment
        if chars[i] == '#' {
            tokens.push(Token {
                text: line[offset..].to_string(),
                kind: TokenKind::Comment,
            });
            break;
        }

        // String literal (" or ') — check triple-quote first
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];

            // Triple-quoted string (""" or ''')
            if i + 2 < len && chars[i + 1] == quote && chars[i + 2] == quote {
                let mut s = String::new();
                s.push(quote);
                s.push(quote);
                s.push(quote);
                i += 3;
                loop {
                    if i + 2 < len && chars[i] == quote && chars[i + 1] == quote && chars[i + 2] == quote {
                        s.push(quote);
                        s.push(quote);
                        s.push(quote);
                        i += 3;
                        break;
                    }
                    if i >= len {
                        break; // unterminated triple-quote (continues on next line)
                    }
                    if chars[i] == '\\' && i + 1 < len {
                        s.push(chars[i]);
                        s.push(chars[i + 1]);
                        i += 2;
                    } else {
                        s.push(chars[i]);
                        i += 1;
                    }
                }
                tokens.push(Token { text: s, kind: TokenKind::String });
                continue;
            }

            // Single-quoted string
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

        // Word (identifier, keyword, decorator)
        if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == '@' {
            let is_decorator = chars[i] == '@';
            let start = i;
            if is_decorator {
                i += 1;
            }
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let kind = if is_decorator {
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

        // Punctuation
        let start = i;
        while i < len
            && !chars[i].is_alphabetic()
            && chars[i] != '_'
            && !chars[i].is_ascii_digit()
            && chars[i] != '"'
            && chars[i] != '\''
            && chars[i] != '#'
            && chars[i] != '@'
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
    c"python".as_ptr()
}

#[no_mangle]
pub extern "C" fn file_extensions() -> *const std::ffi::c_char {
    c"py,pyw".as_ptr()
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
        format!("def {}(", word),
        format!("async def {}(", word),
    ];
    let class_patterns = [
        format!("class {}:", word),
        format!("class {}(", word),
    ];
    let var_patterns = [
        format!("{}: ", word),
        format!("{} =", word),
    ];

    for line in file_content.lines() {
        let trimmed = line.trim();
        for pat in &fn_patterns {
            if trimmed.starts_with(pat.as_str()) {
                let sig = trimmed.trim_end_matches(':').trim_end();
                return Some(format!("```python\n{sig}\n```"));
            }
        }
        for pat in &class_patterns {
            if trimmed.starts_with(pat.as_str()) {
                let sig = trimmed.trim_end_matches(':').trim_end();
                return Some(format!("```python\n{sig}\n```"));
            }
        }
        for pat in &var_patterns {
            if trimmed.starts_with(pat.as_str()) {
                let end = trimmed.find('=').unwrap_or(trimmed.len());
                let sig = trimmed[..end].trim_end();
                return Some(format!("```python\n{sig}\n```"));
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
    Some(("pylsp".to_string(), vec![]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword() {
        let tokens = tokenize_line("def main():");
        assert!(tokens.iter().any(|t| t.text == "def" && t.kind == TokenKind::Keyword));
        assert!(tokens.iter().any(|t| t.text == "main" && t.kind == TokenKind::Function));
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize_line("# this is a comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_inline_comment() {
        let tokens = tokenize_line("x = 1  # inline comment");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Comment));
    }

    #[test]
    fn test_string() {
        let tokens = tokenize_line(r#"s = "hello""#);
        assert!(tokens.iter().any(|t| t.text == "\"hello\"" && t.kind == TokenKind::String));
    }

    #[test]
    fn test_number() {
        let tokens = tokenize_line("x = 42");
        assert!(tokens.iter().any(|t| t.text == "42" && t.kind == TokenKind::Number));
    }

    #[test]
    fn test_decorator() {
        let tokens = tokenize_line("@staticmethod");
        assert!(tokens.iter().any(|t| t.text == "@staticmethod" && t.kind == TokenKind::Macro));
    }

    #[test]
    fn test_type_keyword() {
        let tokens = tokenize_line("x: int = 0");
        assert!(tokens.iter().any(|t| t.text == "int" && t.kind == TokenKind::KeywordType));
    }

    #[test]
    fn test_triple_double_quote() {
        let tokens = tokenize_line(r#"x = """hello world""""#);
        assert!(tokens.iter().any(|t| t.text == r#""""hello world""""# && t.kind == TokenKind::String));
    }

    #[test]
    fn test_triple_single_quote() {
        let tokens = tokenize_line("x = '''docstring'''");
        assert!(tokens.iter().any(|t| t.text == "'''docstring'''" && t.kind == TokenKind::String));
    }

    #[test]
    fn test_triple_quote_unterminated() {
        let tokens = tokenize_line(r#"x = """this continues"#);
        assert!(tokens.iter().any(|t| t.kind == TokenKind::String));
    }
}
