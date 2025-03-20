build:
    cc *.c -o main

# test everything
test-all: build
    ./test --all

# run a specific test
test TEST: build
    ./test --test {{TEST}}

# use Powershell instead of sh:
set shell := ["powershell.exe", "-c"]

hello:
  Write-Host "Hello, world!"

# use cmd.exe instead of sh:
set shell := ["cmd.exe", "/c"]

list:
  dir

recipe-name:
    echo 'This is a recipe!'

# this is a comment
another-recipe:
    @echo 'This is another recipe.'

publish:
    cargo test
    # tests passed, time to publish!
    cargo publish

build:
    cc main.c foo.c bar.c -o main

test: build
    ./test

sloc:
    @echo "`wc -l *.c` lines of code"

test:
  echo 'Testing!'

build:
  echo 'Building!'

default:
  @just --list

alias b := build

build:
  echo 'Building!'

set shell := ["zsh", "-cu"]

foo:
  # this line will be run as `zsh -cu 'ls **/*.txt'`
  ls **/*.txt

set NAME

set NAME := true

set export

a := "hello"

@foo b:
  echo $a
  echo $b

set positional-arguments

@foo bar:
  echo $0
  echo $1

# use python3 to execute recipe lines and backticks
set shell := ["python3", "-c"]

# use print to capture result of evaluation
foos := `print("foo" * 4)`

foo:
  print("Snake snake snake snake.")
  print("{{foos}}")

set shell := ["python3", "-c"]

set shell := ["bash", "-uc"]

set shell := ["zsh", "-uc"]

set shell := ["fish", "-c"]

# build stuff
build:
  ./bin/build

# test stuff
test:
  ./bin/test

version := "0.2.7"
tardir  := "awesomesauce-" + version
tarball := tardir + ".tar.gz"

publish:
    rm -f {{tarball}}
    mkdir {{tardir}}
    cp README.md *.c {{tardir}}
    tar zcvf {{tarball}} {{tardir}}
    scp {{tarball}} me@server.com:release/
    rm -rf {{tarball}} {{tardir}}

braces:
    echo 'I {{{{LOVE}} curly braces!'

braces:
    echo '{{'I {{LOVE}} curly braces!'}}'

braces:
    echo 'I {{ "{{" }}LOVE}} curly braces!'

string-with-tab             := "\t"
string-with-newline         := "\n"
string-with-carriage-return := "\r"
string-with-double-quote    := "\""
string-with-slash           := "\\"
string-with-no-newline      := "\
"

single := '
hello
'

double := "
goodbye
"

escapes := '\t\n\r\"\\'

# this string will evaluate to `foo\nbar\n`
x := '''
  foo
  bar
'''

# this string will evaluate to `abc\n  wuv\nbar\n`
y := """
  abc
    wuv
  xyz
"""

foo:
    -cat foo
    echo 'Done!'

march := arch()
mos := os()
mos := os_family()

system-info:
    @echo "This is an {{arch()}} machine".

# - `env_var(key)` – Retrieves the environment variable with name `key`, aborting if it is not present.
# 
# - `env_var_or_default(key, default)` – Retrieves the environment variable with name `key`, returning `default` if it is not present.
# 
# - `invocation_directory()` - Retrieves the path of the current working directory, before `just` changed it (chdir'd) prior to executing commands.

rustfmt:
    find {{invocation_directory()}} -name \*.rs -exec rustfmt {} \;

build:
    cd {{invocation_directory()}}; ./some_script_that_needs_to_be_run_from_here

script:
  ./{{justfile_directory()}}/scripts/some_script

executable:
    @echo The executable is at: {{just_executable()}}

serve:
  @echo "Starting server with database $DATABASE_ADDRESS on port $SERVER_PORT..."
  ./server --database $DATABASE_ADDRESS --port $SERVER_PORT

localhost := `dumpinterfaces | cut -d: -f2 | sed 's/\/.*//' | sed 's/ //g'`

serve:
    ./serve {{localhost}} 8080

# This backtick evaluates the command `echo foo\necho bar\n`, which produces the value `foo\nbar\n`.
stuff := ```
    echo foo
    echo bar
  ```

foo := if "2" == "2" { "Good!" } else { "1984" }

bar:
  @echo "{{foo}}"

foo := if "hello" != "goodbye" { "xyz" } else { "abc" }

bar:
  @echo {{foo}}

foo := if env_var("RELEASE") == "true" { `get-something-from-release-database` } else { "dummy-value" }

bar foo:
  echo {{ if foo == "bar" { "hello" } else { "goodbye" } }}

os := "linux"

test: build
    ./test --test {{os}}

build:
    ./build {{os}}

export RUST_BACKTRACE := "1"

test:
    # will print a stack trace if it crashes
    cargo test

test $RUST_BACKTRACE="1":
    # will print a stack trace if it crashes
    cargo test

export FOO := "world"
# This backtick will fail with "WORLD: unbound variable"
BAR := `echo hello $WORLD`

# Running `just a foo` will fail with "A: unbound variable"
a $A $B=`echo $A`:
  echo $A $B

build target:
    @echo 'Building {{target}}...'
    cd {{target}} && make

default: (build "main")

build target:
  @echo 'Building {{target}}...'
  cd {{target}} && make

default := 'all'

test target tests=default:
    @echo 'Testing {{target}}:{{tests}}...'
    ./test --tests {{tests}} {{target}}

arch := "wasm"

test triple=(arch + "-unknown-unknown"):
  ./test {{triple}}

backup +FILES:
  scp {{FILES}} me@server.com:

commit MESSAGE *FLAGS:
  git commit {{FLAGS}} -m "{{MESSAGE}}"

test +FLAGS='-q':
  cargo test {{FLAGS}}

search QUERY:
    lynx https://www.google.com/?q={{QUERY}}

search QUERY:
    lynx 'https://www.google.com/?q={{QUERY}}'

foo $bar:
  echo $bar

a:
  echo 'A!'

b: a
  echo 'B!'
  just c

c:
  echo 'C!'

polyglot: python js perl sh ruby

python:
    #!/usr/bin/env python3
    print('Hello from python!')

js:
    #!/usr/bin/env node
    console.log('Greetings from JavaScript!')

perl:
    #!/usr/bin/env perl
    print "Larry Wall says Hi!\n";

sh:
    #!/usr/bin/env sh
    hello='Yo'
    echo "$hello from a shell script!"

ruby:
    #!/usr/bin/env ruby
    puts "Hello from ruby!"

foo:
    #!/usr/bin/env bash
    set -euxo pipefail
    hello='Yo'
    echo "$hello from Bash!"

echo:
  #!/bin/sh

  echo "Hello!"

foo:
  x=hello && echo $x # This works!
  y=bye
  echo $y            # This doesn't, `y` is undefined here!

foo:
  #!/usr/bin/env bash
  set -euxo pipefail
  x=hello
  echo $x

foo:
  pwd    # This `pwd` will print the same directory…
  cd bar
  pwd    # …as this `pwd`!

foo:
  cd bar && pwd

foo:
  #!/usr/bin/env bash
  set -euxo pipefail
  cd bar
  pwd

conditional:
    if true; then echo 'True!'; fi

conditional:
    if true; then \
        echo 'True!'; \
    fi

conditional:
    #!/usr/bin/env sh
    if true; then
        echo 'True!'
    fi

for:
    for file in `ls .`; do echo $file; done

for:
    for file in `ls .`; do \
        echo $file; \
    done

for:
    #!/usr/bin/env sh
    for file in `ls .`; do
        echo $file
    done

while:
    while `server-is-dead`; do ping -c 1 server; done

while:
    while `server-is-dead`; do \
        ping -c 1 server; \
    done

while:
    #!/usr/bin/env sh
    while `server-is-dead`; do
        do ping -c 1 server
    done

test: _test-helper
  ./bin/test

_test-helper:
  ./bin/super-secret-test-helper-stuff


@quiet:
  echo hello
  echo goodbye
  @# all done!

foo:
  #!/usr/bin/env bash
  echo 'Foo!'

@bar:
  #!/usr/bin/env bash
  echo 'Bar!'

default:
  @just --choose

test:
  ./test
