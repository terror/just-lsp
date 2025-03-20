set shell := ["pwsh", "-c"]
set dotenv-filename := ".env-local"
set dotenv-load

recipe:
	Write-Host "Hello, world!"
	Get-ChildItem -Path C:\
