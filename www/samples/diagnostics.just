unused := 'Try removing this variable'
home := env_var('HOME')

build target: prepare
  @echo '{{home}}/{{targte}}'

greet:
  @echo '{{uppercaes("hello")}}'

test:
  cargo test

test:
  cargo test --release
