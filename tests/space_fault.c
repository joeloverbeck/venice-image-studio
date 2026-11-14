/* Synthetic filesystem boundary for the running-app test, never shipped. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdlib.h>
#include <sys/statvfs.h>
#include <unistd.h>

int statvfs(const char *path, struct statvfs *buf) {
    int (*real_call)(const char *, struct statvfs *) = dlsym(RTLD_NEXT, "statvfs");
    int result = real_call(path, buf);
    const char *marker = getenv("VENICE_TEST_SPACE_MARKER");
    if (!result && marker && !access(marker, F_OK)) buf->f_bavail = 0;
    return result;
}

int statvfs64(const char *path, struct statvfs64 *buf) {
    int (*real_call)(const char *, struct statvfs64 *) = dlsym(RTLD_NEXT, "statvfs64");
    int result = real_call(path, buf);
    const char *marker = getenv("VENICE_TEST_SPACE_MARKER");
    if (!result && marker && !access(marker, F_OK)) buf->f_bavail = 0;
    return result;
}
