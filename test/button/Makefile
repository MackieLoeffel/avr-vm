.PHONY: clean all install
.SUFFIXES:.c .elf

all: button.elf button.dis button.bin

clean:
	rm -f button.elf button.dis button.bin

button.elf: button.c ../libspicboard/libspicboard.a

include ../libspicboard/debug.mk
