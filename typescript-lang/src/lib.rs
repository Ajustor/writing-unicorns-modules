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
    "abstract",
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "declare",
    "default",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "module",
    "namespace",
    "new",
    "null",
    "of",
    "override",
    "private",
    "protected",
    "public",
    "readonly",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "var",
    "void",
    "while",
];

const TYPE_KEYWORDS: &[&str] = &[
    "any", "bigint", "boolean", "never", "number", "object", "string", "symbol", "unknown",
];

fn char_byte_offset(chars: &[char], char_idx: usize) -> usize {
    chars[..char_idx].iter().map(|c| c.len_utf8()).sum()
}

/// Tokenize a single line of TypeScript source code.
pub fn tokenize_line(line: &str) -> Vec<Token> {
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

        // String / template literal
        if chars[i] == '"' || chars[i] == '\'' || chars[i] == '`' {
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

        // Non-word punctuation
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
    c"typescript".as_ptr()
}

#[no_mangle]
pub extern "C" fn file_extensions() -> *const std::ffi::c_char {
    c"ts,tsx".as_ptr()
}

/// Tokenize a line of TypeScript code. Returns JSON: [{"text":"...","kind":"keyword"}, ...]
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

/// Scan TypeScript source text for a definition of `word` and return a code-fenced signature.
/// Returns `None` if nothing is found.
pub fn hover_info(word: &str, file_content: &str) -> Option<String> {
    let fn_patterns = [
        format!("function {}(", word),
        format!("function {} (", word),
        format!("async function {}(", word),
        format!("async function {} (", word),
    ];
    let arrow_patterns = [
        format!("const {} = (", word),
        format!("const {} = async (", word),
        format!("let {} = (", word),
        format!("var {} = (", word),
    ];
    let decl_patterns = [
        format!("class {} ", word),
        format!("class {}{}", word, '{'),
        format!("interface {} ", word),
        format!("interface {}{}", word, '{'),
        format!("type {} =", word),
        format!("const {}", word),
        format!("let {}", word),
        format!("var {}", word),
    ];

    for line in file_content.lines() {
        let trimmed = line.trim();
        for pat in &fn_patterns {
            if trimmed.contains(pat.as_str()) {
                let sig = trimmed.trim_end_matches('{').trim_end();
                return Some(format!("```ts\n{sig}\n```"));
            }
        }
        for pat in &arrow_patterns {
            if trimmed.starts_with(pat.as_str()) {
                let sig = trimmed.trim_end_matches('{').trim_end();
                return Some(format!("```ts\n{sig}\n```"));
            }
        }
        for pat in &decl_patterns {
            if trimmed.starts_with(pat.as_str()) {
                let end = trimmed
                    .find('=')
                    .or_else(|| trimmed.find(';'))
                    .unwrap_or(trimmed.len());
                let sig = trimmed[..end].trim_end();
                return Some(format!("```ts\n{sig}\n```"));
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

/// Returns the LSP server command for TypeScript: typescript-language-server
pub fn lsp_server_command() -> Option<(String, Vec<String>)> {
    Some(("typescript-language-server".to_string(), vec!["--stdio".to_string()]))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword() {
        let tokens = tokenize_line("const x = 1;");
        assert!(tokens
            .iter()
            .any(|t| t.text == "const" && t.kind == TokenKind::Keyword));
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize_line("// this is a comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_string_double_quote() {
        let tokens = tokenize_line(r#"const s = "hello";"#);
        assert!(tokens
            .iter()
            .any(|t| t.text == "\"hello\"" && t.kind == TokenKind::String));
    }

    #[test]
    fn test_string_single_quote() {
        let tokens = tokenize_line("const s = 'world';");
        assert!(tokens
            .iter()
            .any(|t| t.text == "'world'" && t.kind == TokenKind::String));
    }

    #[test]
    fn test_template_literal() {
        let tokens = tokenize_line("const s = `hello ${name}`;");
        assert!(tokens
            .iter()
            .any(|t| t.kind == TokenKind::String && t.text.starts_with('`')));
    }

    #[test]
    fn test_number() {
        let tokens = tokenize_line("const x = 42;");
        assert!(tokens
            .iter()
            .any(|t| t.text == "42" && t.kind == TokenKind::Number));
    }

    #[test]
    fn test_type_annotation() {
        let tokens = tokenize_line("const x: number = 5;");
        assert!(tokens
            .iter()
            .any(|t| t.text == "number" && t.kind == TokenKind::KeywordType));
    }

    #[test]
    fn test_interface() {
        let tokens = tokenize_line("interface Foo {");
        assert!(tokens
            .iter()
            .any(|t| t.text == "interface" && t.kind == TokenKind::Keyword));
    }

    #[test]
    fn test_class() {
        let tokens = tokenize_line("class Foo extends Bar {");
        assert!(tokens
            .iter()
            .any(|t| t.text == "class" && t.kind == TokenKind::Keyword));
        assert!(tokens
            .iter()
            .any(|t| t.text == "extends" && t.kind == TokenKind::Keyword));
    }

    #[test]
    fn test_function() {
        let tokens = tokenize_line("function greet() {");
        assert!(tokens
            .iter()
            .any(|t| t.text == "function" && t.kind == TokenKind::Keyword));
        assert!(tokens
            .iter()
            .any(|t| t.text == "greet" && t.kind == TokenKind::Function));
    }

    #[test]
    fn test_any_type() {
        let tokens = tokenize_line("let x: any = null;");
        assert!(tokens
            .iter()
            .any(|t| t.text == "any" && t.kind == TokenKind::KeywordType));
    }
}
