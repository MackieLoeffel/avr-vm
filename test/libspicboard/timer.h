#ifndef TIMER_H
#define TIMER_H
#include <stdint.h>

/**  
 * \addtogroup Timer Timer module
 *
 * \brief The timer module provides an event interface to the hardware timers.
 *
 * The module uses the 16-bit <b>timer 1</b> of the ATmega32. The timer is dynamically
 * configured as needed by the registered alarms and should always be clocked
 * as slow as possible to keep the interrupt load low. When no alarms are
 * registered, the timer clock is disabled.
 *
 * \note The timer module uses dynamic memory management (malloc()/free()) for the allocation
 * of the ALARM types. This is also done from within ISRs. Thus care must be taken when
 * calling malloc()/free() with interrupts enabled.
 *
 * \note Interrupts must be enabled for the timer to work.
 *
 * @{
 * \file timer.h
 * \version \$Rev: 7715 $
 */

/**
 * \brief ALARM type
 * This is type of a struct containing information about an alarm.
 */
typedef struct ALARM ALARM;

/**
 * \brief Type for alarm callback functions.
 *
 * Alarm callback functions are invoked on the interrupt
 * level whenever the associated alarm expires. The
 * programming model for callback functions is similar
 * to that of interrupt service routines.
 */
typedef void (*alarmcallback_t) (void);

/**
 * \brief Cancel an alarm.
 *
 * \param alrm identifier of the alarm that should be canceled
 * \retval  0  success
 * \retval -1  an error occurred
 * \sa sb_timer_setAlarm
 * \note alarms must not be canceled twice
 */
int8_t sb_timer_cancelAlarm(ALARM *alrm);

/**
 * \brief Create a new alarm.
 *
 * This function can be used to set single shot, as well as repeating timers.
 * 
 * - <b>Single shot:</b> Set cycle to 0. This alarm <b>must not</b> be canceled after being fired.
 * - <b>Repetitive:</b> The first shot can be adjusted be setting alarmtime > 0. Otherwise cycle is used.
 *
 * \note The callback function is called from within the ISR-context.
 *
 * \param callback  callback function that will be invoked whenever the alarm expires
 * \param alarmtime time in ms relative to the current time when the alarm shall expire the first time.
 *                    If set to 0 cycle time will be used.
 * \param cycle     time in ms for alarms that periodically expire after the first regular expiry.
 *                     Set to 0 for single shot timers.
 * \return the identifier of the alarm, or NULL if creating the alarm failed.
 *
 * \warning  Canceling a timer twice or canceling a single shot timer after its expiry may
 *	       cause unexpected results.
 *
 * \sa  sb_timer_cancelAlarm
 */
ALARM *sb_timer_setAlarm(alarmcallback_t callback, uint16_t alarmtime, uint16_t cycle);

/**
 *
 *  \brief waits for a specific number of ms
 *
 * This function must not be invoked with interrupts disabled, i.e. from an interrupt
 * handler (or generally, from the ISR level) or a critical section of the application.
 *
 * The CPU is set in sleep mode while waiting.
 *
 * \param waittime wait time in ms 
 * \retval  0  success
 * \retval -1  alarm could not be activated
 * \retval -2  sb_timer_delay invoked while interrupts disabled
 * \sa sb_timer_delay_abort
 *
 */
int8_t sb_timer_delay(uint16_t waittime);

/**
 * \brief Aborts an active sb_timer_delay.
 *
 * This function must be invoked on the ISR level.
 *
 * \sa sb_timer_delay
 */
void sb_timer_delay_abort();

/** @}*/

#endif

