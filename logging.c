#include <stdarg.h>
#include <stdio.h>

extern void rust_log(const char* msg);

void xcb_log_wrapper(const char *fmt, ...) {
    char buf[512];
    va_list argp;
    va_start(argp, fmt);
    vsnprintf(buf, sizeof(buf), fmt, argp);
    va_end(argp);
    rust_log(buf);
}
