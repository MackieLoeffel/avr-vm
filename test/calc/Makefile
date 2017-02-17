.PHONY: clean all install
.SUFFIXES:.c .elf

all: calc.elf calc.dis calc.bin calc

clean:
	rm -f calc.elf calc.dis calc.bin calc

calc.elf: calc.c ../libspicboard/libspicboard.a

calc: calc-native.c
	gcc $< -o calc

include ../libspicboard/debug.mk
