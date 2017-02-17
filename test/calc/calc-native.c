#include <inttypes.h>
#include <stdio.h>

uint32_t
sum(uint32_t n)
{
	uint32_t s;
	uint32_t i;

	s = 0;
	for (i = 0; i <= n; i++) {
		s += i;
	}

	return s;
}

uint32_t
fac(uint32_t n)
{
	if (n == 0) {
		return 1;
	} else {
		return n * fac(n - 1);
	}
}

void
dump_char(char c)
{
	putchar(c);
    fflush(stdout);
}

void
dump_str(const char *str)
{
	while (*str != '\0') {
		dump_char(*str++);
	}
}

void
_dump_uint32_t(uint32_t x)
{
	if (x != 0) {
		_dump_uint32_t(x / 10);
		dump_char('0' + x % 10);
	}
}

void
dump_uint32_t(uint32_t x)
{
	if (x == 0) {
		dump_char('0');
	} else {
		_dump_uint32_t(x);
	}
}

void
main(void)
{
	uint32_t i;

	for (i = 0; i < 10; i++) {
		dump_str("sum(");
		dump_uint32_t(i);
		dump_str(") = ");
		dump_uint32_t(sum(i));
		dump_str("\n");

		dump_str("fac(");
		dump_uint32_t(i);
		dump_str(") = ");
		dump_uint32_t(fac(i));
		dump_str("\n");
	}

	for (;;);
}
