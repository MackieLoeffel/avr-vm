#include <stdint.h>

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
	for (;;) {
		for (int i = 0; i < 8; i++) {
			sb_led_set_all_leds(1 << i);
			wait(100);
			sb_led_set_all_leds(0x00);
		}
		for (int i = 0; i < 8; i++) {
			sb_led_on(i);
			wait(100);
		}
		for (int i = 0; i < 8; i++) {
			sb_led_set_all_leds(~(1 << i));
			wait(100);
			sb_led_set_all_leds(0xff);
		}
		for (int i = 0; i < 8; i++) {
			sb_led_off(i);
			wait(100);
		}
	}
}
