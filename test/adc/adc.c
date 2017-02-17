#include <stdint.h>

#include <adc.h>
#include <led.h>

static void
wait(uint16_t usec)
{
	for (volatile int32_t i = 0; i < usec * 1000; i++) {
	}
}

void
main(void)
{
	for (;;) {
		uint16_t level0;
		uint16_t level1;

		level0 = sb_adc_read(POTI) >> 8;
		level1 = sb_adc_read(PHOTO) >> 8;

		sb_led_set_all_leds((1 << level0) | (1 << (level1 + 4)));

		wait(1000);
	}
}
