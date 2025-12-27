#include <stdint.h>
#include <stdio.h>

/*
uint16_t r0;
uint16_t r1;
uint16_t r7;
*/

/*
uint16_t better_fn6049() {
	if (a == 0) {
		a = b + 1;
		return a;
	} else if (b == 0) {
		a--;
		b = c;
		return better_fn6049(a, b);
	} else {
		uint16_t tmp = a;
		// mutate b
		b--;
		b = better_fn6049(a, b);
		a = tmp - 1;
		return better_fn6049();
	}
}
*/

uint16_t fn6049(uint16_t r0, uint16_t r1, uint16_t r7) {
	// 6049
	if (r0 == 0) {
		// 6052
		r0 = (r1 + 1) % 32768;
		// 6056
		return r0;
	}

	// 6057
	if (r1 == 0) {
		// 6060
		r0 = (r0 + 32767) % 32768; // dec by 1
		// 6064
		r1 = r7;
		// 6067
		return fn6049(r0, r1, r7);
		// 6069
//		return;
	}

	// 6070 - push r0 on the stack
	uint16_t tmp = r0;
	// 6072
	r1 = (r1 + 32767) % 32768; // decrement by 1
	// 6076
	r0 = fn6049(r0, r1, r7);
	// 6078
	r1 = r0;
	// 6081 - pop
	r0 = tmp;
	// 6083
	r0 = (r0 + 32767) % 32768; // decrement by 1
	// 6087
	r0 = fn6049(r0, r1, r7);
	// 6089
	return r0;
}
/*
6049 jt (32768 != 0 -> 6057)
6052 add <0> = 32769 + 1
6056 ret
6057 jt (32769 != 0 -> 6070)
6060 add <0> = 32768 + 32767
6064 set <1> = 32775
6067 call
6069 ret
6070 push 32768
6072 add <1> = 32769 + 32767
6076 call
6078 set <1> = 32768
6081 pop
6083 add <0> = 32768 + 32767
6087 call
6089 ret
*/

int main() {
	for (uint16_t i = 1; i < 32768; i++) {
		printf("trying to solve r7=%u\n", i);
		uint16_t value = fn6049(4, 1, i);
		if (value == 6) {
			printf("it worked!\n");
		}
	}

	printf("done\n");
	return 0;
}
