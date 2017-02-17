#include <stdint.h>

#include <button.h>
#include <led.h>

static void
wait(uint16_t usec)
{
	for (volatile uint32_t i = 0; i < usec * 1000; i++) {
	}
}

void
main(void)
{
	uint8_t b0 = 0;
	uint8_t b1 = 0;
	uint8_t s0 = 0;
	uint8_t s1 = 0;

	for (;;) {
		if (! b0
		 && sb_button_getState(BUTTON0) == BTNPRESSED) {
			s0 ^= 1;
			b0 = 1;
		}
		if (b0
		 && sb_button_getState(BUTTON0) == BTNRELEASED) {
			b0 = 0;
		}
		if (! b1
		 && sb_button_getState(BUTTON1) == BTNPRESSED) {
			s1 ^= 1;
			b1 = 1;
		}
		if (b1
		 && sb_button_getState(BUTTON1) == BTNRELEASED) {
			b1 = 0;
		}

		sb_led_set_all_leds(0xf0 * s0 | 0x0f * s1);

		wait(100);
	}
}
