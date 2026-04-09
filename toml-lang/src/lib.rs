#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind { Keyword, KeywordType, String, Comment, Number, Function, Macro, Normal, Property }

impl TokenKind {
    pub fn color_category(&self) -> &'static str {
        match self {
            TokenKind::Keyword => "keyword", TokenKind::KeywordType => "type",
            TokenKind::String => "string", TokenKind::Comment => "comment",
            TokenKind::Number => "number", TokenKind::Function => "function",
            TokenKind::Macro => "macro", TokenKind::Normal => "normal",
            TokenKind::Property => "property",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token { pub text: String, pub kind: TokenKind }

pub fn tokenize_line(line: &str) -> Vec<Token> {
    let trimmed = line.trim_start();

    // Full-line comment
    if trimmed.starts_with('#') {
        return vec![Token { text: line.to_string(), kind: TokenKind::Comment }];
    }

    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    // Table header: [section] or [[array]]
    if trimmed.starts_with('[') {
        tokens.push(Token { text: line.to_string(), kind: TokenKind::Macro });
        return tokens;
    }

    // Key = value: detect key before '='
    if let Some(eq_pos) = chars.iter().position(|&c| c == '=') {
        // Key part (before =)
        let key: String = chars[..eq_pos].iter().collect();
        let key_trimmed = key.trim();
        if !key_trimmed.is_empty() && key_trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '"' || c == ' ') {
            tokens.push(Token { text: key.clone(), kind: TokenKind::Property });
            tokens.push(Token { text: "=".to_string(), kind: TokenKind::Normal });
            i = eq_pos + 1;
        }
    }

    while i < len {
        // Inline comment
        if chars[i] == '#' {
            tokens.push(Token { text: chars[i..].iter().collect(), kind: TokenKind::Comment });
            break;
        }

        // Triple-quoted string
        if (chars[i] == '"' || chars[i] == '\'') && i + 2 < len && chars[i+1] == chars[i] && chars[i+2] == chars[i] {
            let quote = chars[i];
            let mut s = String::new();
            s.push(quote); s.push(quote); s.push(quote);
            i += 3;
            loop {
                if i + 2 < len && chars[i] == quote && chars[i+1] == quote && chars[i+2] == quote {
                    s.push(quote); s.push(quote); s.push(quote);
                    i += 3;
                    break;
                }
                if i >= len { break; }
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(Token { text: s, kind: TokenKind::String });
            continue;
        }

        // String
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            let mut s = String::new();
            s.push(quote);
            i += 1;
            while i < len {
                let c = chars[i];
                s.push(c);
                i += 1;
                if c == '\\' && i < len { s.push(chars[i]); i += 1; }
                else if c == quote { break; }
            }
            tokens.push(Token { text: s, kind: TokenKind::String });
            continue;
        }

        // Number
        if chars[i].is_ascii_digit() || (chars[i] == '+' || chars[i] == '-') && i + 1 < len && chars[i+1].is_ascii_digit() {
            let mut s = String::new();
            while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_' || chars[i] == '+' || chars[i] == '-' || chars[i] == ':' || chars[i] == 'T' || chars[i] == 'Z') {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(Token { text: s, kind: TokenKind::Number });
            continue;
        }

        // Word
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '-') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let kind = match word.as_str() {
                "true" | "false" => TokenKind::Keyword,
                "inf" | "nan" => TokenKind::KeywordType,
                _ => TokenKind::Normal,
            };
            tokens.push(Token { text: word, kind });
            continue;
        }

        // Whitespace/punctuation
        let start = i;
        while i < len && !chars[i].is_alphanumeric() && chars[i] != '_' && chars[i] != '"' && chars[i] != '\'' && chars[i] != '#' && !(chars[i] == '+' || chars[i] == '-') {
            i += 1;
        }
        if i > start {
            tokens.push(Token { text: chars[start..i].iter().collect(), kind: TokenKind::Normal });
        }
        if i == start { i += 1; } // safety
    }

    tokens
}

// ── FFI ──────────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn language_id() -> *const std::ffi::c_char { c"toml".as_ptr() }

#[no_mangle]
pub extern "C" fn file_extensions() -> *const std::ffi::c_char { c"toml".as_ptr() }

#[no_mangle]
pub unsafe extern "C" fn tokenize_line_ffi(line_ptr: *const std::ffi::c_char) -> *mut std::ffi::c_char {
    let line = unsafe { std::ffi::CStr::from_ptr(line_ptr).to_str().unwrap_or("") };
    let tokens = tokenize_line(line);
    let json = tokens_to_json(&tokens);
    std::ffi::CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { unsafe { drop(std::ffi::CString::from_raw(ptr)) } }
}

pub fn hover_info(_word: &str, _file_content: &str) -> Option<String> { None }

#[no_mangle]
pub unsafe extern "C" fn hover_info_ffi(_word_ptr: *const std::ffi::c_char, _file_content_ptr: *const std::ffi::c_char) -> *mut std::ffi::c_char {
    std::ptr::null_mut()
}

pub fn lsp_server_command() -> Option<(String, Vec<String>)> {
    Some(("taplo".to_string(), vec!["lsp".to_string(), "stdio".to_string()]))
}

fn tokens_to_json(tokens: &[Token]) -> String {
    let parts: Vec<String> = tokens.iter().map(|t| {
        format!(r#"{{"text":{},"kind":"{}"}}"#, serde_json_escape(&t.text), t.kind.color_category())
    }).collect();
    format!("[{}]", parts.join(","))
}

fn serde_json_escape(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment() {
        let tokens = tokenize_line("# this is a comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_table_header() {
        let tokens = tokenize_line("[package]");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Macro));
    }

    #[test]
    fn test_key_value() {
        let tokens = tokenize_line("name = \"hello\"");
        assert!(tokens.iter().any(|t| t.text.contains("name") && t.kind == TokenKind::Property));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::String));
    }

    #[test]
    fn test_boolean() {
        let tokens = tokenize_line("enabled = true");
        assert!(tokens.iter().any(|t| t.text == "true" && t.kind == TokenKind::Keyword));
    }

    #[test]
    fn test_number() {
        let tokens = tokenize_line("port = 8080");
        assert!(tokens.iter().any(|t| t.text == "8080" && t.kind == TokenKind::Number));
    }
}
