#include <linux/ashmem.h>
#include <fcntl.h>
#include <sys/ioctl.h>
#include <string.h>

int ashmem_open(const char *name, size_t size) {
    int fd = open("/dev/ashmem", O_RDWR);
    if (fd == -1) { return fd; }
    if (ioctl(fd, ASHMEM_SET_NAME, name) == -1) {
        return -1;
    }
    if (ioctl(fd, ASHMEM_SET_SIZE, size) == -1) {
        return -1;
    }
    return fd;
}

const char *ashmem_get_name(int fd, char *buf, size_t size) {
    char name[ASHMEM_NAME_LEN];
    if (ioctl(fd, ASHMEM_GET_NAME, name) == -1) {
        return NULL;
    }
    size_t n = strlen(name);
    if (n < size) {
        size = n;
    }
    strncpy(buf, name, size);
    return buf;
}

size_t ashmem_get_size(int fd) {
    size_t size = 0;
    if (ioctl(fd, ASHMEM_GET_SIZE, &size) == -1) {
        return 0;
    }
    return size;
}
