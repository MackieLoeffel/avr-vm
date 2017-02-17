#include <timer.h>
#include <avr/interrupt.h>
#include <button.h>
#include <7seg.h>
#include <led.h>
#include <adc.h>

static volatile enum { PO=1, LI=0, COUNT=3 } mode = PO;

void switch_mode() {
	switch(mode) {
		case PO:
			mode = LI;
			sb_7seg_showStr("LI");
			break;
		case LI:
			mode = COUNT;
			break;
		case COUNT:
			mode = PO;
			sb_7seg_showStr("PO");
			break;
	}
}

void main(void) {
	/* uint16_t level; */
	/* int8_t zahl = 99; */

	sb_7seg_showStr("PO");
    for(;;) {}
/* 	sb_button_registerListener(BUTTON1, BTNPRESSED, switch_mode); */
/* 	sb_button_registerListener(BUTTON0, BTNPRESSED, switch_mode); */
/* 	sei(); */

/* 	while(42){ */
/* 		level = sb_adc_read(mode & 1); */

/* 		switch(mode) { */
/* 			case PO: */
/* 			case LI: */
/* 				sb_led_show_level(level>>2, 0xff); */
/* 				sb_timer_delay(10); */
/* 				break; */


/* 			case COUNT: */
/* 				sb_led_show_level(level>>2, 0xff); */

/* 				zahl= (zahl==0)?99:zahl-1; */
/* 				sb_7seg_showNumber(zahl); */

/* 				sb_timer_delay(1060-level); /\* leave a few ms to keep it observable (sort of) *\/ */
/* 				break; */
/* 		} */
/* 	} */
}
