# Test that numbers are parsed as errors but do not mess up the syntax

a := 100
# <- variable
# ^^ operator
#    ^^^ error

foo:
# <- function
#  ^ operator

b := "a" + 1.234
# <- variable
# ^^ operator
#    ^^^ string
#        ^ operator
#          ^^^^ error

bar:
# <- function
#  ^ operator
