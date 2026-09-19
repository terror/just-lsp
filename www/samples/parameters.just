greet name='world':
  @echo 'Hello, {{name}}!'

build target profile='debug':
  @echo 'Building {{target}} with profile {{profile}}'

test *args:
  cargo test {{args}}

format +files:
  rustfmt {{files}}

serve $PORT='8080':
  python3 -m http.server "$PORT"
