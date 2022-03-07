ROOT_DIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

include $(ROOT_DIR)/remarkable.mk

TARGET := unix

DEPS = gcc-multilib llvm clang-11
noop=
space = $(noop) $(noop)

ifeq ($(TARGET),remarkable)
	export CARGO_BUILD_TARGET = armv7-unknown-linux-gnueabihf
	CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER = $(REMARKABLE_TOOLCHAIN_PATH)/sysroots/x86_64-codexsdk-linux/usr/bin/arm-remarkable-linux-gnueabi/arm-remarkable-linux-gnueabi-gcc
	export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS += -C link-arg=-march=armv7-a
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS += -C link-arg=-marm
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS += -C link-arg=-mfpu=neon
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS += -C link-arg=-mfloat-abi=hard
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS += -C link-arg=-mcpu=cortex-a9
	CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS += -C link-arg=--sysroot=$(REMARKABLE_TOOLCHAIN_PATH)/sysroots/cortexa7hf-neon-remarkable-linux-gnueabi
	export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS
endif

deps: install-toolchain
	sudo apt install $(DEPS)

ifeq ($(TARGET),remarkable)
build:
	/bin/sh $(REMARKABLE_TOOLCHAIN_PATH)/environment-setup-cortexa7hf-neon-remarkable-linux-gnueabi; \
	cargo build
else
build:
	cargo build
endif