#ifndef _7SEG_H
#define _7SEG_H

#include <stdint.h> 


/** 
 *
 * \addtogroup 7seg 7SEG (Seven Segment Display)
 *
 * \brief Controls the two 7-segment displays on the board.
 *
 * The two 7-segment displays of the SPiCboard share one common
 * port of the MCU. The two displays can be connected and
 * disconnected from the port using two transistors. By quickly
 * and periodically connecting and disconnecting the displays
 * an observer will not be able to notice when a display is
 * disabled and both displays can be used apparently simultaneously.
 * \note As the timer-library is used, interrupts must be enabled for the
 * display to work
 * \sa timer.h
 * @{
 * \file 7seg.h
 * \version \$Rev: 9414 $
 */

/**
 * \brief prints a number in the range [-9; 99] on the 7-segment display
 *
 *
 * \param nmbr the number to print
 * \retval  0  success
 * \retval -1  nmbr is smaller than -9
 * \retval -2  nmbr is greater than 99
 */
int8_t sb_7seg_showNumber(int8_t nmbr);

/**
 * \brief prints the hexadecimal representation of an 8-bit unsigned integer on the 7-segment display
 *
 * \param nmbr the number to print
 * \retval 0 on success
 * \retval !0 on error
 */
int8_t sb_7seg_showHexNumber(uint8_t nmbr);

/**
 * \brief prints a 2 character string on the 7-segment display
 *
 * Supported characters are in the group [-_ 0-9A-Za-z] (contains space).
 * Read <a href="http://en.wikipedia.org/wiki/Seven-segment_display_character_representations">this</a>
 * article for possible representations of these characters. Two
 * characters of the set should never have the same representation.
 * No differentiation is made between upper- and lowercase characters.
 *
 * \param str the 0-terminated string
 *
 * \retval  0  success
 * \retval -1  character at position 0 not printable
 * \retval -2  character at position 1 not printable
 * \retval -3  both characters not printable
 * \retval -4  str is an empty string
 *  
 */
int8_t sb_7seg_showStr(const char *str);

/**
 *  \brief disables the 7-segment displays
 *
 *  Any running alarms are unregistered.
 */
void sb_7seg_disable(void);

/** @}*/

#endif

