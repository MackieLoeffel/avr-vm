.PHONY: clean all install
.SUFFIXES:.c .elf

all: jump.elf jump.dis jump.bin jump jump-time.bin

clean:
	rm -f jump.elf jump.dis jump.bin jump

jump.elf: jump.c ../libspicboard/libspicboard.a

jump: jump-native.c
	gcc  $< -o jump

include ../libspicboard/debug.mk
