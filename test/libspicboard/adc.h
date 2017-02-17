#ifndef ADC_H
#define ADC_H

#include <stdint.h>


/** 
 * \addtogroup ADC ADC (Analog to Digital Converter)
 * \brief Interface to the AD-converter of the ATmega32, which allows to
 *   query the potentiometer and the photosensor of the board.
 *
 * @{
 * \file adc.h
 * \version \$Rev: 7715 $
 */

/**
 * \brief device ids of available periphery connected to ADC channels.
 */
typedef enum {
	PHOTO = 0, /**< the photosensor (brighter ambience yields higher numbers) **/
	POTI = 1   /**< the potentiometer (rotation towards LEDs yields higher numbers) **/
} ADCDEV;

/**
 *  \brief perform a 10-bit A/D conversion for a specific channel/device.
 *
 *  \param dev id of a device connected to the ADC
 *
 *  \retval >0  10-bit result of the conversion
 *	\retval -1  invalid device id
 */
int16_t sb_adc_read(ADCDEV dev);

/** @}*/

#endif

