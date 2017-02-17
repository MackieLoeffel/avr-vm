.PHONY: clean all install
.SUFFIXES:.c .elf

all: simple.elf simple.dis simple.bin simple-time.bin

clean:
	rm -f simple.elf simple.dis simple.bin

simple.elf: simple.c ../libspicboard/libspicboard.a

include ../libspicboard/debug.mk
