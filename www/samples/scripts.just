set unstable
set script-interpreter := ['bash', '-eu']

message := 'Hello from just'

[script]
shell:
  for name in world just; do
    echo "Hello, $name!"
  done

python:
  #!/usr/bin/env python3
  message = '{{message}}'
  for word in message.split():
      print(word.upper())

[script: 'python3']
count limit='3':
  for number in range(int('{{limit}}')):
      print(number)
