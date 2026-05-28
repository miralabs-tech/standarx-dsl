; Highlighting queries for the standarx DSL.
;
; Captures use the standard tree-sitter highlight names so editors
; (nvim-treesitter, Helix, Zed, GitHub Linguist) map them onto their
; own colour schemes without custom configuration.

; Identifiers — by context
(block kind: (identifier) @keyword)
(block label: (plain_string) @string.special)
(assignment key: (identifier) @property)
(map key: (identifier) @property)

; Reference paths
(ref (identifier) @variable)

; Literals
(boolean) @boolean
(null) @constant.builtin
(integer) @number
(float) @number.float
(plain_string) @string
(template_inline) @string
(template_multiline) @string

; Escapes
(escape_sequence) @string.escape

; Interpolation
(interpolation "${" @punctuation.special)
(interpolation "}" @punctuation.special)
(interpolation (identifier) @variable)
(interpolation (ref (identifier) @variable))

; Punctuation
"{" @punctuation.bracket
"}" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"," @punctuation.delimiter
"." @punctuation.delimiter

; Comments
(comment) @comment
