export KDIST_DIR ?= "${HOME}/tools/kdist-assets"

default: which-all

test: which
	komet test

prove: which
	komet prove

which:
	@which komet
	@echo "KDIST_DIR=${KDIST_DIR}"

which-all:
	@echo "You have the following komet binaries on your PATH"
	@which -a komet
