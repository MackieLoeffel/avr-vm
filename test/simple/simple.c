#include <inttypes.h>
#include <avr/io.h>

static void
print(const char *str)
{
	while (*str != '\0') {
		UDR = *str++;
	}
}

static void
wait(uint32_t ms)
{
	volatile uint32_t i;

	for (i = 0; i < ms << 4; i++) {
	}
}

void
main(void)
{
	for (;;) {
		print("Hallo VM!\n");
		wait(1000);
	}
}
