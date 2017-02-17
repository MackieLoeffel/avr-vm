#ifndef BUTTON_H
#define BUTTON_H

#include <stdint.h>


/** 

 *
 * \addtogroup Button
 *
 *  \brief The button module enables event-driven and polling access
 *   to the buttons of the SPiCboard.
 *
 * The SPiCboard is equipped with two buttons. Button 0 is debounced in
 * hardware, whereas Button 1 needs to be debounced in software by the
 * button module. Debouncing is transparent to the application, that
 * can use both buttons through the provided interface without the
 * need to care about debouncing.
 *
 * The debouncing code makes use of the timer module. When no listeners
 * are registered for Button 1, the debouncing code is disabled and all
 * alarms registered at the timer should be canceled.
 *
 * The button module uses dynamic memory management to maintain the
 * listener queues.
 * 
 * @{
 * \file button.h
 * \version \$Rev: 7715 $
 */

/**
 *	\brief Identifiers for all available buttons.
 *
 */
typedef enum {
	BUTTON0 = 4, /**< Button 0 */
	BUTTON1 = 8  /**< Button 1 */
} BUTTON;

/**
 *  \brief Events for buttons.
 *
 * Pressed and released events for buttons. 
 */
typedef enum {
	BTNPRESSED = 1, /**< Button was pressed */
	BTNRELEASED = 2 /**< Button was released */
} BUTTONEVENT;

/**
 *  \brief Type for button event callback functions.
 *
 * A button callback function is called on the interrupt level whenever
 * an event at a button occurs that the function was registered for.
 * The callback function is passed the button id and the type of event
 * that occurred. This way, the same callback function can be registered
 * for different buttons and events.
 */
typedef void (*buttoncallback_t) (BUTTON, BUTTONEVENT);

/**
 *  \brief Register a callback function for a button event.
 *
 *  \param btn      the id of the button
 *  \param eve      the type of event that the callback function should be invoked for.
 *                  event types can be bitwise or'd to register a callback for both
 *                  pressed and released events.
 *  \param callback pointer to the callback function. This function is called from the
 *  				interrupt handler.
 *  \retval 0  success,
 *  \retval !0 error
 *  \sa sb_button_unregisterListener 
 *	\sa buttoncallback_t
 */
int8_t sb_button_registerListener(BUTTON btn, BUTTONEVENT eve, buttoncallback_t callback);

/**
 *  \brief Unregister a callback function for a button event.
 *
 *
 *  \param btn      the id of the button
 *  \param eve      the type of event that the callback function should be invoked for.
 *                  event types can be bitwise or'd to register a callback for both
 *                  pressed and released events.
 *  \param callback pointer to the callback function
 *  \return 0 on success, negative value on error
 *  \retval  0  success
 *  \retval -1  the callback function was not registered with the given button/event combination
 *  \sa         sb_button_registerListener
 *
 */
int8_t sb_button_unregisterListener(BUTTON btn, BUTTONEVENT eve, buttoncallback_t callback);

/**
 *  \brief Query the current state of a button.
 *
 *  \param btn  id of the button
 *  \return The buttons current state (pressed or released) as a \ref BUTTONEVENT
 */
BUTTONEVENT sb_button_getState(BUTTON btn);

/** @}*/

#endif

