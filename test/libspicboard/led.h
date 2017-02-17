#ifndef LED_H
#define LED_H
#include <stdint.h>


/**
 *
 * \addtogroup LED LED access
 * 
 *  \brief Interface to the board's 8 LEDs
 *
 * @{
 *
 * \file led.h
 *
 *  \version \$Rev: 7715 $
 *
 */

/**
 *  \brief LED identifiers
 *
 */
typedef enum {
	RED0=0, YELLOW0=1, GREEN0=2, BLUE0=3,
	RED1=4, YELLOW1=5, GREEN1=6, BLUE1=7
} LED;

/**
 *  \brief Activates a specific LED
 *
 *  \param led LED ID
 *  \return 0 on success, negative value on error
 *   \retval  0  success
 *   \retval -1  invalid LED ID
 */
int8_t sb_led_on(LED led);

/**
 *  \brief Deactivates a specific LED
 *
 *  \param led LED ID
 *  \return 0 on success, negative value on error
 *   \retval  0  success
 *   \retval -1  invalid LED ID
 */
int8_t sb_led_off(LED led);

/**
 *  \brief Toggles a specific LED
 *
 *  \param led LED ID
 *  \return 0 on success, negative value on error
 *   \retval  0  success
 *   \retval -1  invalid LED ID
 */
int8_t sb_led_toggle(LED led);

/**
 *  \brief Uses the LED array as a level indicator
 *
 * Allows the array of LEDs to be used as a (fill) level, progress
 * or similar indicator. The 8 LEDs are used to display a ratio of
 * a max-value<=255 in 9 steps.
 *
 *  \param level level value
 *  \param max   maximum possible value
 *  \return the number of LEDs turned on on success, negative value on error
 *   \retval >=0 success
 *   \retval -1  level exceeds max
 *   \retval -2  max is 0
 */
int8_t sb_led_show_level(uint8_t level, uint8_t max);

/**
 *  \brief Sets all LEDs according to a bitfield
 *
 * The bitfield contains one bit for each LED. A set bit enables
 * and a cleared bit disables the corresponding LED.
 *
 *  \param setting  8-bit bitfield describing the desired LED states
 */
void sb_led_set_all_leds(uint8_t setting);

/** @}*/

#endif

