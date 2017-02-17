.PHONY: clean all install
.SUFFIXES:.c .elf

all: seg7.elf seg7.dis seg7.bin

clean:
	rm -f seg7.elf seg7.dis seg7.bin

seg7.elf: seg7.c ../libspicboard/libspicboard.a

include ../libspicboard/debug.mk
