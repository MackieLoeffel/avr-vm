.PHONY: clean all install
.SUFFIXES:.c .elf

all: boardtest.elf boardtest.dis boardtest.bin boardtest-test.elf boardtest-test.dis boardtest-test.bin

clean:
	rm -f boardtest.elf boardtest.dis boardtest.bin

boardtest.elf: boardtest.c ../libspicboard/libspicboard.a

include ../libspicboard/debug.mk
