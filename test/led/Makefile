.PHONY: clean all install
.SUFFIXES:.c .elf

all: led.elf led.dis led.bin

clean:
	rm -f led.elf led.dis led.bin

led.elf: led.c ../libspicboard/libspicboard.a

include ../libspicboard/debug.mk
