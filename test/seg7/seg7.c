#include <stdint.h>

#include <avr/io.h>

static uint8_t seg[2];

static void
show(void)
{
	if ((PORTD >> PD1) & 1) {
		PORTD |= 1 << PD0;
		PORTB = seg[1];
		PORTD &= ~(1 << PD1);
	} else {
		PORTD |= 1 << PD1;
		PORTB = seg[0];
		PORTD &= ~(1 << PD0);
	}
}

static void
show_num(uint8_t num)
{
	static const uint8_t leds[] = {
		0x84, 0x9F, 0xC8, 0x8A, 0x93, 0xA2, 0xA0, 0x8F,
		0x80, 0x82, 0x81, 0xB0, 0xE4, 0x98, 0xE0, 0xE1,
	};

	seg[0] = leds[num / 10];
	seg[1] = leds[num % 10];
}

static void
wait(uint16_t usec)
{
        for (volatile uint32_t i = 0; i < usec * 100; i++) {
		show();
	}
}

void
main(void)
{
	uint8_t count = 0;

	DDRB = 0xff;
	DDRD |= (1 << PD0) | (1 << PD1);

	for (;;) {
		show_num(count);
		wait(1000);
		count = (count == 99) ? 0 : count + 1;
	}
}
