REMARKABLE_TOOLCHAIN_PATH = $(CURDIR)/3rdparty/codex/rm11x/3.1.15
TOOLCHAIN_INSTALLER_SH = $(CURDIR)/toolchains/codex-x86_64-cortexa7hf-neon-rm11x-toolchain-3.1.15.sh

.PHONY: install-toolchain

install-toolchain: $(REMARKABLE_TOOLCHAIN_PATH)

$(REMARKABLE_TOOLCHAIN_PATH):
	$(TOOLCHAIN_INSTALLER_SH) -y -d $(REMARKABLE_TOOLCHAIN_PATH)