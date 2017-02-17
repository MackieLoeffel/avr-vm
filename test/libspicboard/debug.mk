LIBSPICBOARDDIR ?= ../libspicboard

CC   := avr-gcc
SIZE := avr-size
GDB  := avr-gdb

# Warning! -fwhole-program breaks the linkage of additional files
COMMONCFLAGS=-ffreestanding -mmcu=atmega32 -DF_CPU=1000000  -Wall -Werror -pedantic -pedantic-errors -I$(LIBSPICBOARDDIR)

COMMONCFLAGS+=-std=c99

# Use these for debugging...
CFLAGS ?= -g -O0 $(COMMONCFLAGS)

# ...or these for an optimized code image
#CFLAGS ?= -Os $(COMMONCFLAGS)

LDFLAGS ?= -L$(LIBSPICBOARDDIR) -lspicboard

%.bin: %.elf
	avr-objcopy -O binary -R .eeprom $< $@

%.dis: %.elf
	avr-objdump -d $< > $@

%.elf: %.c
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)
	$(SIZE) $@
