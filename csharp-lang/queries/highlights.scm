;; Catch-all: every identifier gets @variable as fallback
(identifier) @variable

;; ─── Methods & Functions ─────────────────────────────────────────────────────

(method_declaration name: (identifier) @function)
(local_function_statement name: (identifier) @function)

;; ─── Types ───────────────────────────────────────────────────────────────────

(interface_declaration name: (identifier) @type)
(class_declaration name: (identifier) @type)
(enum_declaration name: (identifier) @type)
(struct_declaration (identifier) @type)
(record_declaration (identifier) @type)
(namespace_declaration name: (identifier) @type)

(generic_name (identifier) @type)
(type_parameter (identifier) @type)
(parameter type: (identifier) @type)
(type_argument_list (identifier) @type)
(as_expression right: (identifier) @type)
(is_expression right: (identifier) @type)

(constructor_declaration name: (identifier) @constructor)
(destructor_declaration name: (identifier) @constructor)

(_ type: (identifier) @type)

(base_list (identifier) @type)

(predefined_type) @type.builtin

;; Object creation: new MyClass(), new List<T>()
(object_creation_expression type: (identifier) @type)
(object_creation_expression type: (generic_name (identifier) @type))

;; Using directives: using System.Collections.Generic;
(using_directive (identifier) @type)
(using_directive (qualified_name (identifier) @type))

;; Type constraints: where T : IDisposable
(type_parameter_constraints_clause (identifier) @type)

;; ─── Enum Members ────────────────────────────────────────────────────────────

(enum_member_declaration (identifier) @property)

;; ─── Properties & Member Access ──────────────────────────────────────────────

(property_declaration name: (identifier) @property)
(member_access_expression name: (identifier) @property)

;; ─── Literals ────────────────────────────────────────────────────────────────

[
  (real_literal)
  (integer_literal)
] @number

[
  (character_literal)
  (string_literal)
  (raw_string_literal)
  (verbatim_string_literal)
  (interpolated_string_expression)
  (interpolation_start)
  (interpolation_quote)
] @string

(escape_sequence) @string.escape

[
  (boolean_literal)
  (null_literal)
] @constant.builtin

;; ─── Comments ────────────────────────────────────────────────────────────────

(comment) @comment

;; ─── Punctuation ─────────────────────────────────────────────────────────────

[
  ";"
  "."
  ","
] @punctuation.delimiter

[
  "--" "-" "-="
  "&" "&=" "&&"
  "+" "++" "+="
  "<" "<=" "<<" "<<="
  "=" "=="
  "!" "!="
  "=>"
  ">" ">=" ">>" ">>="
  ">>>" ">>>="
  "|" "|=" "||"
  "?" "??" "??="
  "^" "^="
  "~"
  "*" "*=" "/" "/=" "%" "%="
  ":"
] @operator

[
  "(" ")" "[" "]" "{" "}"
  (interpolation_brace)
] @punctuation.bracket

;; ─── Keywords ────────────────────────────────────────────────────────────────

[
  (modifier)
  "this"
  (implicit_type)
] @keyword

[
  "add" "alias" "as" "base" "break" "case" "catch" "checked" "class"
  "continue" "default" "delegate" "do" "else" "enum" "event"
  "explicit" "extern" "finally" "for" "foreach" "global" "goto"
  "if" "implicit" "interface" "is" "lock" "namespace" "notnull"
  "operator" "params" "return" "remove" "sizeof" "stackalloc"
  "static" "struct" "switch" "throw" "try" "typeof" "unchecked"
  "using" "while" "new" "await" "in" "yield" "get" "set"
  "when" "out" "ref" "from" "where" "select" "record" "init"
  "with" "let"
] @keyword

;; ─── Attributes ──────────────────────────────────────────────────────────────

(attribute name: (identifier) @attribute)

;; ─── Parameters ──────────────────────────────────────────────────────────────

(parameter name: (identifier) @variable.parameter)

;; ─── Function Calls (AFTER property patterns to override member access) ──────

(invocation_expression (identifier) @function)
(invocation_expression (member_access_expression name: (identifier) @function))
