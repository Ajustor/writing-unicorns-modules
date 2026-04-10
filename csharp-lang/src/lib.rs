use std::cell::RefCell;

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
    Property,
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
            TokenKind::Property => "property",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub kind: TokenKind,
}

// ── Multi-line state ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum LineState {
    Normal,
    InBlockComment,
    InVerbatimString,
    InRawStringLiteral(u8), // count of opening quotes (""" = 3, etc.)
}

thread_local! {
    static STATE: RefCell<LineState> = const { RefCell::new(LineState::Normal) };
}

fn get_state() -> LineState {
    STATE.with(|s| *s.borrow())
}

fn set_state(new: LineState) {
    STATE.with(|s| *s.borrow_mut() = new);
}

// ── Keywords ────────────────────────────────────────────────────────────────

const KEYWORDS: &[&str] = &[
    "abstract", "as", "base", "break", "case", "catch", "checked", "class",
    "const", "continue", "default", "delegate", "do", "else", "enum", "event",
    "explicit", "extern", "false", "finally", "fixed", "for", "foreach", "goto",
    "if", "implicit", "in", "interface", "internal", "is", "lock", "namespace",
    "new", "null", "operator", "out", "override", "params", "private", "protected",
    "public", "readonly", "ref", "return", "sealed", "sizeof", "stackalloc",
    "static", "struct", "switch", "this", "throw", "true", "try", "typeof",
    "unchecked", "unsafe", "using", "virtual", "void", "volatile", "while",
    "async", "await", "var", "dynamic", "yield", "when", "where", "nameof",
    "record", "init", "required", "with", "not", "and", "or",
];

const TYPE_KEYWORDS: &[&str] = &[
    "bool", "byte", "sbyte", "char", "decimal", "double", "float", "int", "uint",
    "long", "ulong", "short", "ushort", "string", "object", "nint", "nuint",
    "String", "Int32", "Int64", "Boolean", "Double", "Single", "Decimal",
    "Object", "Byte", "Char", "DateTime", "TimeSpan", "Guid",
    "List", "Dictionary", "IEnumerable", "IList", "IDictionary", "ICollection",
    "Task", "ValueTask", "Action", "Func", "Nullable", "Span", "Memory",
    "HashSet", "Queue", "Stack", "LinkedList", "SortedList",
];

// ── Tokenizer ───────────────────────────────────────────────────────────────

pub fn tokenize_line(line: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut state = get_state();

    // ── Resume multi-line block comment ─────────────────────────────────
    if state == LineState::InBlockComment {
        let start = 0;
        while i + 1 < len {
            if chars[i] == '*' && chars[i + 1] == '/' {
                i += 2;
                state = LineState::Normal;
                break;
            }
            i += 1;
        }
        if state == LineState::InBlockComment {
            // Entire line is inside comment
            i = len;
        }
        tokens.push(Token {
            text: chars[start..i].iter().collect(),
            kind: TokenKind::Comment,
        });
        if state == LineState::InBlockComment {
            set_state(state);
            return tokens;
        }
    }

    // ── Resume multi-line verbatim string (@"...") ──────────────────────
    if state == LineState::InVerbatimString {
        let start = 0;
        while i < len {
            if chars[i] == '"' {
                if i + 1 < len && chars[i + 1] == '"' {
                    i += 2; // escaped quote
                } else {
                    i += 1; // end of verbatim string
                    state = LineState::Normal;
                    break;
                }
            } else {
                i += 1;
            }
        }
        tokens.push(Token {
            text: chars[start..i].iter().collect(),
            kind: TokenKind::String,
        });
        if state == LineState::InVerbatimString {
            set_state(state);
            return tokens;
        }
    }

    // ── Resume multi-line raw string literal (""" ... """) ──────────────
    if let LineState::InRawStringLiteral(quote_count) = state {
        let start = 0;
        let closing: String = "\"".repeat(quote_count as usize);
        if let Some(pos) = find_substr(&chars, i, &closing) {
            i = pos + quote_count as usize;
            state = LineState::Normal;
        } else {
            i = len;
        }
        tokens.push(Token {
            text: chars[start..i].iter().collect(),
            kind: TokenKind::String,
        });
        if state != LineState::Normal {
            set_state(state);
            return tokens;
        }
    }

    // ── Normal tokenization ─────────────────────────────────────────────

    // Full-line single-line comment
    let trimmed = &chars[i..];
    if trimmed.len() >= 2
        && trimmed.iter().take_while(|c| c.is_whitespace()).count() + 2 <= trimmed.len()
    {
        let skip = trimmed.iter().take_while(|c| c.is_whitespace()).count();
        if skip + 1 < trimmed.len() && trimmed[skip] == '/' && trimmed[skip + 1] == '/' {
            tokens.push(Token {
                text: chars[i..].iter().collect(),
                kind: TokenKind::Comment,
            });
            set_state(LineState::Normal);
            return tokens;
        }
    }

    while i < len {
        // Block comment /* ... */
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            let mut closed = false;
            while i + 1 < len {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    closed = true;
                    break;
                }
                i += 1;
            }
            if !closed {
                i = len;
                state = LineState::InBlockComment;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::Comment,
            });
            continue;
        }

        // Inline comment //
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            tokens.push(Token {
                text: chars[i..].iter().collect(),
                kind: TokenKind::Comment,
            });
            break;
        }

        // Raw string literal """ (C# 11)
        if i + 2 < len && chars[i] == '"' && chars[i + 1] == '"' && chars[i + 2] == '"' {
            let start = i;
            let mut q_count: u8 = 0;
            while i < len && chars[i] == '"' {
                q_count += 1;
                i += 1;
            }
            // Look for closing sequence of same count
            let closing: String = "\"".repeat(q_count as usize);
            if let Some(pos) = find_substr(&chars, i, &closing) {
                i = pos + q_count as usize;
            } else {
                i = len;
                state = LineState::InRawStringLiteral(q_count);
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::String,
            });
            continue;
        }

        // Verbatim interpolated $@"..." or @$"..."
        if i + 2 < len
            && ((chars[i] == '$' && chars[i + 1] == '@' && chars[i + 2] == '"')
                || (chars[i] == '@' && chars[i + 1] == '$' && chars[i + 2] == '"'))
        {
            let start = i;
            i += 3;
            let mut closed = false;
            while i < len {
                if chars[i] == '"' {
                    if i + 1 < len && chars[i + 1] == '"' {
                        i += 2; // escaped
                    } else {
                        i += 1;
                        closed = true;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            if !closed {
                state = LineState::InVerbatimString;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::String,
            });
            continue;
        }

        // Verbatim string @"..."
        if chars[i] == '@' && i + 1 < len && chars[i + 1] == '"' {
            let start = i;
            i += 2;
            let mut closed = false;
            while i < len {
                if chars[i] == '"' {
                    if i + 1 < len && chars[i + 1] == '"' {
                        i += 2; // escaped
                    } else {
                        i += 1;
                        closed = true;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            if !closed {
                state = LineState::InVerbatimString;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::String,
            });
            continue;
        }

        // Interpolated string $"..."
        if chars[i] == '$' && i + 1 < len && chars[i + 1] == '"' {
            let start = i;
            i += 2;
            while i < len {
                let c = chars[i];
                i += 1;
                if c == '\\' && i < len {
                    i += 1;
                } else if c == '"' {
                    break;
                }
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::String,
            });
            continue;
        }

        // Regular string "..."
        if chars[i] == '"' {
            let start = i;
            i += 1;
            while i < len {
                let c = chars[i];
                i += 1;
                if c == '\\' && i < len {
                    i += 1;
                } else if c == '"' {
                    break;
                }
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::String,
            });
            continue;
        }

        // Char literal '...'
        if chars[i] == '\'' {
            let start = i;
            i += 1;
            while i < len {
                let c = chars[i];
                i += 1;
                if c == '\\' && i < len {
                    i += 1;
                } else if c == '\'' {
                    break;
                }
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::String,
            });
            continue;
        }

        // Number
        if chars[i].is_ascii_digit()
            || (chars[i] == '.' && i + 1 < len && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            if chars[i] == '0' && i + 1 < len && (chars[i + 1] == 'x' || chars[i + 1] == 'X') {
                i += 2;
                while i < len && (chars[i].is_ascii_hexdigit() || chars[i] == '_') {
                    i += 1;
                }
            } else {
                while i < len
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_')
                {
                    i += 1;
                }
            }
            // Suffix
            while i < len && matches!(chars[i], 'f' | 'F' | 'd' | 'D' | 'm' | 'M' | 'L' | 'U' | 'u' | 'l') {
                i += 1;
            }
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::Number,
            });
            continue;
        }

        // Attribute [Attr]
        if chars[i] == '[' && i + 1 < len && chars[i + 1].is_alphabetic() {
            let start = i;
            let mut j = i + 1;
            let mut depth = 1;
            while j < len && depth > 0 {
                if chars[j] == '[' { depth += 1; }
                if chars[j] == ']' { depth -= 1; }
                j += 1;
            }
            if depth == 0 {
                tokens.push(Token {
                    text: chars[start..j].iter().collect(),
                    kind: TokenKind::Macro,
                });
                i = j;
                continue;
            }
        }

        // Preprocessor directive
        if chars[i] == '#' && (i == 0 || chars[..i].iter().all(|c| c.is_whitespace())) {
            tokens.push(Token {
                text: chars[i..].iter().collect(),
                kind: TokenKind::Macro,
            });
            break;
        }

        // Word (identifier/keyword)
        if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == '@' {
            let start = i;
            if chars[i] == '@' {
                i += 1;
            }
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let bare_word = word.trim_start_matches('@');
            let kind = if TYPE_KEYWORDS.contains(&bare_word) {
                TokenKind::KeywordType
            } else if KEYWORDS.contains(&bare_word) {
                TokenKind::Keyword
            } else if i < len && chars[i] == '(' {
                TokenKind::Function
            } else if i < len && chars[i] == '<' {
                TokenKind::KeywordType
            } else {
                TokenKind::Normal
            };
            tokens.push(Token { text: word, kind });
            continue;
        }

        // Punctuation / operators
        let start = i;
        while i < len
            && !chars[i].is_alphanumeric()
            && !matches!(chars[i], '_' | '"' | '\'' | '/' | '#' | '@' | '$' | '[' | '.')
        {
            i += 1;
        }
        if i > start {
            tokens.push(Token {
                text: chars[start..i].iter().collect(),
                kind: TokenKind::Normal,
            });
        }
        if i == start {
            tokens.push(Token {
                text: chars[i].to_string(),
                kind: TokenKind::Normal,
            });
            i += 1;
        }
    }

    set_state(state);
    tokens
}

/// Find a substring in a char slice starting at `from`.
fn find_substr(chars: &[char], from: usize, needle: &str) -> Option<usize> {
    let needle_chars: Vec<char> = needle.chars().collect();
    let nlen = needle_chars.len();
    if nlen == 0 || from + nlen > chars.len() {
        return None;
    }
    for pos in from..=chars.len() - nlen {
        if chars[pos..pos + nlen] == needle_chars[..] {
            return Some(pos);
        }
    }
    None
}

// ── FFI ─────────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn language_id() -> *const std::ffi::c_char {
    c"csharp".as_ptr()
}

#[no_mangle]
pub extern "C" fn file_extensions() -> *const std::ffi::c_char {
    c"cs,csx".as_ptr()
}

/// Reset the multi-line tokenizer state. Call before tokenizing a new document.
#[no_mangle]
pub extern "C" fn reset_tokenizer() {
    set_state(LineState::Normal);
}

#[no_mangle]
pub unsafe extern "C" fn tokenize_line_ffi(
    line_ptr: *const std::ffi::c_char,
) -> *mut std::ffi::c_char {
    let line = unsafe { std::ffi::CStr::from_ptr(line_ptr).to_str().unwrap_or("") };
    let tokens = tokenize_line(line);
    let json = tokens_to_json(&tokens);
    std::ffi::CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() {
        unsafe { drop(std::ffi::CString::from_raw(ptr)) }
    }
}

pub fn hover_info(word: &str, file_content: &str) -> Option<String> {
    let patterns = [
        format!("class {} ", word),
        format!("class {}:", word),
        format!("class {}<", word),
        format!("interface {} ", word),
        format!("interface {}:", word),
        format!("interface {}<", word),
        format!("struct {} ", word),
        format!("struct {}:", word),
        format!("enum {} ", word),
        format!("record {} ", word),
        format!("record {}<", word),
        format!(" {}(", word),
        format!("void {}(", word),
        format!("async {}(", word),
        format!("Task {}(", word),
        format!("Task<{}", word),
        format!("static {}(", word),
    ];

    for line in file_content.lines() {
        let trimmed = line.trim();
        for pat in &patterns {
            if trimmed.contains(pat.as_str()) {
                let sig = trimmed
                    .trim_end_matches('{')
                    .trim_end_matches("=>")
                    .trim_end();
                return Some(format!("```csharp\n{sig}\n```"));
            }
        }
    }
    None
}

#[no_mangle]
pub unsafe extern "C" fn hover_info_ffi(
    word_ptr: *const std::ffi::c_char,
    file_content_ptr: *const std::ffi::c_char,
) -> *mut std::ffi::c_char {
    let word = unsafe { std::ffi::CStr::from_ptr(word_ptr).to_str().unwrap_or("") };
    let content = unsafe {
        std::ffi::CStr::from_ptr(file_content_ptr)
            .to_str()
            .unwrap_or("")
    };
    match hover_info(word, content) {
        Some(s) => std::ffi::CString::new(s)
            .map(|c| c.into_raw())
            .unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

pub fn lsp_server_command() -> Option<(String, Vec<String>)> {
    Some(("OmniSharp".to_string(), vec!["-lsp".to_string()]))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        set_state(LineState::Normal);
    }

    #[test]
    fn test_keyword() {
        reset();
        let tokens = tokenize_line("public class MyClass {");
        assert!(tokens.iter().any(|t| t.text == "public" && t.kind == TokenKind::Keyword));
        assert!(tokens.iter().any(|t| t.text == "class" && t.kind == TokenKind::Keyword));
    }

    #[test]
    fn test_type_keyword() {
        reset();
        let tokens = tokenize_line("int x = 0;");
        assert!(tokens.iter().any(|t| t.text == "int" && t.kind == TokenKind::KeywordType));
    }

    #[test]
    fn test_single_line_comment() {
        reset();
        let tokens = tokenize_line("// this is a comment");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
    }

    #[test]
    fn test_inline_block_comment() {
        reset();
        let tokens = tokenize_line("/* block */ int x;");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Comment));
        assert!(tokens.iter().any(|t| t.text == "int" && t.kind == TokenKind::KeywordType));
    }

    #[test]
    fn test_multi_line_block_comment() {
        reset();
        let t1 = tokenize_line("int a; /* start of comment");
        assert!(t1.iter().any(|t| t.kind == TokenKind::Comment));
        assert!(t1.iter().any(|t| t.text == "int" && t.kind == TokenKind::KeywordType));

        let t2 = tokenize_line("  still in comment");
        assert_eq!(t2.len(), 1);
        assert_eq!(t2[0].kind, TokenKind::Comment);

        let t3 = tokenize_line("end of comment */ int b;");
        assert!(t3.iter().any(|t| t.kind == TokenKind::Comment));
        assert!(t3.iter().any(|t| t.text == "int" && t.kind == TokenKind::KeywordType));
    }

    #[test]
    fn test_multi_line_verbatim_string() {
        reset();
        let t1 = tokenize_line(r#"var s = @"line one"#);
        assert!(t1.iter().any(|t| t.kind == TokenKind::String));

        let t2 = tokenize_line(r#"line two";"#);
        assert!(t2.iter().any(|t| t.kind == TokenKind::String));
        // After closing quote, state should be normal
        assert_eq!(get_state(), LineState::Normal);
    }

    #[test]
    fn test_interpolated_string() {
        reset();
        let tokens = tokenize_line(r#"var s = $"Hello {name}";"#);
        assert!(tokens.iter().any(|t| t.kind == TokenKind::String && t.text.starts_with("$\"")));
    }

    #[test]
    fn test_raw_string_literal() {
        reset();
        let t1 = tokenize_line(r#"var s = """"#);
        assert!(t1.iter().any(|t| t.kind == TokenKind::String));

        let t2 = tokenize_line(r#"  multi-line content"#);
        assert_eq!(t2.len(), 1);
        assert_eq!(t2[0].kind, TokenKind::String);

        let t3 = tokenize_line(r#"  """;"#);
        assert!(t3.iter().any(|t| t.kind == TokenKind::String));
    }

    #[test]
    fn test_string() {
        reset();
        let tokens = tokenize_line(r#"string s = "hello";"#);
        assert!(tokens.iter().any(|t| t.text == "\"hello\"" && t.kind == TokenKind::String));
    }

    #[test]
    fn test_function() {
        reset();
        let tokens = tokenize_line("public void DoSomething() {");
        assert!(tokens.iter().any(|t| t.text == "DoSomething" && t.kind == TokenKind::Function));
    }

    #[test]
    fn test_number() {
        reset();
        let tokens = tokenize_line("int x = 42;");
        assert!(tokens.iter().any(|t| t.text == "42" && t.kind == TokenKind::Number));
    }

    #[test]
    fn test_attribute() {
        reset();
        let tokens = tokenize_line("[Serializable]");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Macro));
    }

    #[test]
    fn test_preprocessor() {
        reset();
        let tokens = tokenize_line("#region MyRegion");
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Macro));
    }
}
