.PHONY: clean all install
.SUFFIXES:.c .elf

all: adc.elf adc.dis adc.bin

clean:
	rm -f adc.elf adc.dis adc.bin

adc.elf: adc.c ../libspicboard/libspicboard.a

include ../libspicboard/debug.mk
