project := 'hello-world'
version := '1.0.0'
profile := env('PROFILE', 'debug')
target := 'target' / profile
artifact := target / (project + '-' + version)
greeting := if os() == 'windows' { 'Hello' } else { 'Howdy' }

export APP_NAME := project

info:
  @echo '{{greeting}} from {{project}} {{version}}'
  @echo 'Building {{artifact}} on {{os()}}/{{arch()}}'

build:
  mkdir -p '{{target}}'
  echo "$APP_NAME" > '{{artifact}}'
