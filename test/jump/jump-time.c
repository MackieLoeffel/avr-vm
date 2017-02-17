#include <inttypes.h>
#include <avr/io.h>
#include <avr/cpufunc.h>

static void
dump_char(char c)
{
	UDR = c;
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
dump_int32_t(int32_t x)
{
	if (x < 0) {
		dump_char('-');
		x = -x;
	}
	dump_uint32_t((uint32_t) x);
}

void
test_uint8_t(uint8_t a, uint8_t b)
{
	if (a < b) {
		dump_uint32_t(a);
		dump_str(" < ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a <= b) {
		dump_uint32_t(a);
		dump_str(" <= ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a > b) {
		dump_uint32_t(a);
		dump_str(" > ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a >= b) {
		dump_uint32_t(a);
		dump_str(" >= ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a == b) {
		dump_uint32_t(a);
		dump_str(" == ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a != b) {
		dump_uint32_t(a);
		dump_str(" != ");
		dump_uint32_t(b);
		dump_str("\n");
	}
}

void
test_uint16_t(uint16_t a, uint16_t b)
{
	if (a < b) {
		dump_uint32_t(a);
		dump_str(" < ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a <= b) {
		dump_uint32_t(a);
		dump_str(" <= ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a > b) {
		dump_uint32_t(a);
		dump_str(" > ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a >= b) {
		dump_uint32_t(a);
		dump_str(" >= ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a == b) {
		dump_uint32_t(a);
		dump_str(" == ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a != b) {
		dump_uint32_t(a);
		dump_str(" != ");
		dump_uint32_t(b);
		dump_str("\n");
	}
}

void
test_uint32_t(uint32_t a, uint32_t b)
{
	if (a < b) {
		dump_uint32_t(a);
		dump_str(" < ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a <= b) {
		dump_uint32_t(a);
		dump_str(" <= ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a > b) {
		dump_uint32_t(a);
		dump_str(" > ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a >= b) {
		dump_uint32_t(a);
		dump_str(" >= ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a == b) {
		dump_uint32_t(a);
		dump_str(" == ");
		dump_uint32_t(b);
		dump_str("\n");
	}
	if (a != b) {
		dump_uint32_t(a);
		dump_str(" != ");
		dump_uint32_t(b);
		dump_str("\n");
	}
}

void
test_int8_t(int8_t a, int8_t b)
{
	if (a < b) {
		dump_int32_t(a);
		dump_str(" < ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a <= b) {
		dump_int32_t(a);
		dump_str(" <= ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a > b) {
		dump_int32_t(a);
		dump_str(" > ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a >= b) {
		dump_int32_t(a);
		dump_str(" >= ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a == b) {
		dump_int32_t(a);
		dump_str(" == ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a != b) {
		dump_int32_t(a);
		dump_str(" != ");
		dump_int32_t(b);
		dump_str("\n");
	}
}

void
test_int16_t(int16_t a, int16_t b)
{
	if (a < b) {
		dump_int32_t(a);
		dump_str(" < ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a <= b) {
		dump_int32_t(a);
		dump_str(" <= ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a > b) {
		dump_int32_t(a);
		dump_str(" > ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a >= b) {
		dump_int32_t(a);
		dump_str(" >= ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a == b) {
		dump_int32_t(a);
		dump_str(" == ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a != b) {
		dump_int32_t(a);
		dump_str(" != ");
		dump_int32_t(b);
		dump_str("\n");
	}
}

void
test_int32_t(int32_t a, int32_t b)
{
	if (a < b) {
		dump_int32_t(a);
		dump_str(" < ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a <= b) {
		dump_int32_t(a);
		dump_str(" <= ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a > b) {
		dump_int32_t(a);
		dump_str(" > ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a >= b) {
		dump_int32_t(a);
		dump_str(" >= ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a == b) {
		dump_int32_t(a);
		dump_str(" == ");
		dump_int32_t(b);
		dump_str("\n");
	}
	if (a != b) {
		dump_int32_t(a);
		dump_str(" != ");
		dump_int32_t(b);
		dump_str("\n");
	}
}

void
main(void)
{
	static volatile int32_t val[] = {
		INT32_MIN, INT32_MIN + 1, INT32_MIN + 2,
		INT16_MIN, INT16_MIN + 1, INT16_MIN + 2,
		INT8_MIN, INT8_MIN + 1, INT8_MIN + 2,
		-2, -1,
		0,
		1, 2,
		INT8_MAX - 2, INT8_MAX - 1, INT8_MAX,
		INT16_MAX - 2, INT16_MAX - 1, INT16_MAX,
		INT32_MAX - 2, INT32_MAX - 1, INT32_MAX,
	};
	int i;
	int j;

	for (i = 0; i < sizeof(val) / sizeof(val[0]); i++) {
		for (j = 0; j < sizeof(val) / sizeof(val[0]); j++) {
			if ((uint8_t) val[i] != val[i]
			 && (uint8_t) val[j] != val[j]) {
				test_uint8_t(val[i], val[j]);
				test_int8_t(val[i], val[j]);
			}
			if ((uint16_t) val[i] != val[i]
			 && (uint16_t) val[j] != val[j]) {
				test_uint16_t(val[i], val[j]);
				test_int16_t(val[i], val[j]);
			}
			test_uint32_t(val[i], val[j]);
			test_int32_t(val[i], val[j]);
		}
	}

    _NOP();

	for (;;);
}
