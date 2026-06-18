# justfile grammar

This is a readable sketch of `grammar.js`. Comments, whitespace, and escaped
newlines may appear between tokens unless a rule says otherwise.

## tokens

````
BACKTICK                         = ` command_body? `
COMMENT                          = #.*
DEDENT                           = emitted when indentation decreases
FORMAT_INDENTED_RAW_STRING       = f'''...'''
FORMAT_INDENTED_STRING           = f"""..."""
FORMAT_RAW_STRING                = f'...'
FORMAT_STRING                    = f"..."
INDENT                           = emitted when indentation increases
INDENTED_BACKTICK                = ``` command_body? ```
INDENTED_RAW_STRING              = '''...'''
INDENTED_STRING                  = """..."""
NAME                             = [a-zA-Z_][a-zA-Z0-9_-]*
NEWLINE                          = \n|\r\n
NUMERIC_ERROR                    = (\d+\.\d*|\d+)
RAW_STRING                       = '[^']*'
SHEBANG                          = #![^\n]*
SHELL_EXPANDED_INDENTED_RAW_STRING = x'''...'''
SHELL_EXPANDED_INDENTED_STRING   = x"""..."""
SHELL_EXPANDED_RAW_STRING        = x'...'
SHELL_EXPANDED_STRING            = x"..."
STRING                           = "..."
TEXT                             = recipe text, only matches in a recipe body
````

`STRING`, `INDENTED_STRING`, `SHELL_EXPANDED_STRING`,
`SHELL_EXPANDED_INDENTED_STRING`, `FORMAT_STRING`, and
`FORMAT_INDENTED_STRING` process `\n`, `\r`, `\t`, `\"`, `\\`, escaped
newlines, and `\u{...}` escapes.

Format strings and command bodies may contain interpolations.

## grammar syntax

```
|   alternation
()  grouping
_?  option (0 or 1 times)
_*  repetition (0 or more times)
_+  repetition (1 or more times)
```

## grammar

```
source_file     : (shebang NEWLINE)? item*

item            : recipe
                | alias
                | assignment
                | eager
                | export
                | unexport
                | function_definition
                | import
                | module
                | setting

function_definition
                : NAME '(' function_parameters? ')' ':=' expression NEWLINE

function_parameters
                : NAME (',' NAME)* ','?

alias           : attribute* 'alias' NAME ':=' (module_path | NAME)

module_path     : NAME ('::' NAME)+

assignment      : attribute* NAME ':=' expression NEWLINE

eager           : attribute* 'eager' assignment

export          : attribute* 'export' assignment

unexport        : attribute* 'unexport' assignment

import          : 'import' '?'? string

module          : attribute* 'mod' '?'? NAME string?

setting         : 'set' NAME (':=' (boolean | string | string_array | expression))? NEWLINE
                | 'set' 'shell' ':=' string_array NEWLINE

boolean         : 'true'
                | 'false'

string_array    : '[' string_array_content? ']'

string_array_content
                : string (',' string)* string?

expression      : '/'? expression_inner

expression_inner
                : if_expression
                | expression_inner '+' expression_inner
                | expression_inner '/' expression_inner
                | expression_inner '&&' expression_inner
                | expression_inner '||' expression_inner
                | value

if_expression   : 'if' condition braced_expression else_if_clause* else_clause?

else_if_clause  : 'else' 'if' condition braced_expression

else_clause     : 'else' braced_expression

braced_expression
                : '{' expression '}'

condition       : expression '==' expression
                | expression '!=' expression
                | expression '=~' (regex_literal | expression)
                | expression

regex_literal   : string

value           : assert_expression
                | function_call
                | external_command
                | NAME
                | string
                | NUMERIC_ERROR
                | '(' expression ')'

assert_expression
                : 'assert' '(' condition (',' expression ','?)? ')'

function_call   : NAME '(' sequence? ')'

external_command
                : BACKTICK
                | INDENTED_BACKTICK

sequence        : expression (',' expression)*

attribute       : '[' attribute_item (',' attribute_item)* ']' NEWLINE

attribute_item  : NAME
                | NAME '(' attribute_argument (',' attribute_argument)* ')'
                | NAME ':' string

attribute_argument
                : expression
                | attribute_named_param

attribute_named_param
                : NAME ('=' expression)?

recipe          : attribute* recipe_header NEWLINE recipe_body?

recipe_header   : '@'? (NAME | 'import') parameters? ':' dependencies?

parameters      : parameter* (parameter | variadic_parameter)

parameter       : '$'? NAME ('=' value)?

variadic_parameter
                : ('*' | '+') parameter

dependencies    : ('&&'? dependency)+

dependency      : module_path
                | NAME
                | dependency_expression

dependency_expression
                : '(' (module_path | NAME) expression* ')'

recipe_body     : INDENT (shebang NEWLINE)? (recipe_line NEWLINE | NEWLINE)* DEDENT

recipe_line     : recipe_line_prefix? (TEXT | interpolation)+

recipe_line_prefix
                : '@-?'
                | '@?-'
                | '-@?'
                | '-?@'
                | '?@-'
                | '?-@'
                | '@-'
                | '@?'
                | '-@'
                | '-?'
                | '?@'
                | '?-'
                | '@'
                | '-'
                | '?'

shebang         : SHEBANG

string          : SHELL_EXPANDED_INDENTED_STRING
                | SHELL_EXPANDED_INDENTED_RAW_STRING
                | SHELL_EXPANDED_STRING
                | SHELL_EXPANDED_RAW_STRING
                | INDENTED_STRING
                | INDENTED_RAW_STRING
                | STRING
                | RAW_STRING
                | format_string

format_string   : FORMAT_STRING
                | FORMAT_INDENTED_STRING
                | FORMAT_RAW_STRING
                | FORMAT_INDENTED_RAW_STRING

command_body    : (interpolation | any_character)+

interpolation   : '{{' expression '}}'
```

Expression operators are left-associative. From highest to lowest precedence,
the infix operators are `+`, `/`, `&&`, and `||`.
