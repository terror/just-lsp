# Injection highlighting shows up as none during tests

localhost := `if [ -f "foo" ]; then; echo hi; fi`
# <- variable
#         ^^ operator
#            ^ punctuation.special
#                                               ^ punctuation.special

# interpolation within injection
foo := `[ {{ dir(localhost) }} = "hi" ] && "bye"`
# <- variable
#   ^^ operator
#      ^ punctuation.special
#         ^^ punctuation.bracket
#            ^^^ function.call
#               ^ punctuation.bracket
#                ^^^^^^^^^ variable
#                         ^ punctuation.bracket
#                           ^^ punctuation.bracket
#                                               ^ punctuation.special

stuff := ```
	echo foo
	echo bar
```
# <- punctuation.special

# These have to be verified manually, no automated tests are available

foo:
	echo "hello {{ foo }}"
	if [ -f "foo" ]; then; \
		echo hi; \
	fi

py_shebang:
	#!/usr/bin/env python
	if foo is not None:
		print("bar")

perl_shebang:
	#!/usr/bin/env perl
	my $iter = $dir->iterator;
	while (my $file = $iter->()) {

	    # See if it is a directory and skip
	    next if $file->is_dir();

	    # Print out the file name and path
	    print "$file\n";
	}
