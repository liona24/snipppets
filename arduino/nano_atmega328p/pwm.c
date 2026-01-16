// Default clock source is internal 8MHz RC oscillator
#define F_CPU 8000000UL

#include <avr/io.h>
#include <util/delay.h>

int main()
{
	// PB0 (D8) for LED blinking, PB2 for PWM (OC1B, Timer 1)
	DDRB |= (1 << PB0) | (1 << PB2);

	//
	// PWM Setup
	//
	//
	// WGM12:0 = 1 1 1 (Fast PWM, TOP = OCRA)
	// COM1A: disconnected, we only need output B
	// COM1B: set OC0B at BOTTOM (non-inverting mode), see table 14-6 of the
	// datasheet
	TCCR1A |= (0 << COM1A1) | (1 << COM1B1) | (1 << WGM11) | (1 << WGM10);
	// Clock select: clkIO / 64
	TCCR1B |= (1 << WGM12) | (1 << CS11) | (1 << CS10);

	// 16Mhz clkIO:
	// With clock select divider 64, this gives us a 25kHz PWM frequency
	OCR1A = 10;
	// Duty cycle [0, 10]
	OCR1B = 1;

	// 
	// Main Loop
	//

	while (1) {
		PORTB |= (1 << PB0);
		_delay_ms(1000);
		PORTB &= ~(1 << PB0);
		_delay_ms(1000);
	}
}
