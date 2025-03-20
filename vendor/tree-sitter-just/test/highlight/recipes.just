#!/use/bin/env just
# <- keyword.directive
# ^^^^^^^^^^^^^^^^^ keyword.directive

foo:
# <- function
#  ^ operator

@bar:
# <- operator
# ^ function
#   ^ operator

baz: foo bar
# <- function
#  ^ operator
#     ^ function.call
#          ^ function.call

qux var1:
# <- function
#   ^^^ variable.parameter
#       ^ operator

quux var *var2:
# <- function
#    ^^^ variable.parameter
#        ^ operator
#          ^^^ variable.parameter
#             ^ operator

corge +quux: baz (quux quux)
# <- function
#     ^ operator
#      ^^^^ variable.parameter
#          ^ operator
#            ^^^ function.call
#                ^ punctuation.bracket
#                 ^^^^ function.call
#                      ^^^ variable
#                          ^ punctuation.bracket

grault abc="def":
# <- function
#      ^^^ variable.parameter
#         ^ operator
#          ^^^^ string
#               ^ operator

garply: foo && bar
# <- function
#     ^ operator
#       ^^^ function.call
#           ^^ operator
#              ^^^ function.call

waldo a="b": foo bar && baz
# <- function
#     ^ variable.parameter
#      ^ operator
#       ^^^ string
#          ^ operator
#            ^^^ function.call
#                ^^^ function.call
#                    ^^ operator
#                       ^^^ function.call

fred: garply && (waldo "x")
# <- function
#     ^^^^^^ function.call
#            ^^ operator
#               ^ punctuation.bracket
#                 ^^^^ function.call
#                      ^^^ string
#                         ^ punctuation.bracket

# plugh
plugh:
  echo "plugh"
# xyzzy
xyzzy:
  echo "xyzzy"

# FIXME: can't test these because we can't place comments between
[private]
[confirm, no-cd]
attributes:
