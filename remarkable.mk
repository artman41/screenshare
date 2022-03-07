ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
REMARKABLE_TOOLCHAIN_PATH = $(ROOT_DIR)/3rdparty/codex/rm11x/3.1.15
TOOLCHAIN_INSTALLER_SH = $(ROOT_DIR)/toolchains/codex-x86_64-cortexa7hf-neon-rm11x-toolchain-3.1.15.sh

.PHONY: install-toolchain

install-toolchain: $(REMARKABLE_TOOLCHAIN_PATH)

$(REMARKABLE_TOOLCHAIN_PATH):
	$(TOOLCHAIN_INSTALLER_SH) -y -d $(REMARKABLE_TOOLCHAIN_PATH)