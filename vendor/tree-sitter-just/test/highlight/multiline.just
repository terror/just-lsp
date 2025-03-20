# Wrapped expression
a := "foo" + \
    "bar"
#   ^^^^^ string

# Smooshed recipes
foo:
    echo foo {{ a }}
bar:
    echo bar {{ os() }}
#               ^^ function.call

# Wrapped dependencies
baz: foo \
    bar
#   ^^^ function.call
