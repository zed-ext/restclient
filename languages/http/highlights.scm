; Comments and request separators (### lines)
[
  (comment)
  (request_separator)
] @comment

; HTTP methods
(method) @keyword

; Target URL
(target_url) @string.special

; Headers
(header
  name: (_) @property)
(header
  ":" @punctuation.delimiter)

; Request bodies
[
  (json_body)
  (xml_body)
  (raw_body)
  (graphql_body)
] @string

; Response handler scripts > {% ... %}
(res_handler_script) @string

; Response redirects >>
(res_redirect) @keyword

; External body references
(external_body) @string.special

; Variable declarations
(variable_declaration
  name: (identifier) @variable)
(variable_declaration
  "=" @punctuation.delimiter)

; Variable references {{ }}
[
  "{{"
  "}}"
] @punctuation.bracket

(variable
  name: (identifier) @variable)
