set shell := ["python3", "-c"]
set dotenv-filename := ".env-local"
set dotenv-load

recipe:
	if True: print(100)
	if "foo" != None: print("foo")
