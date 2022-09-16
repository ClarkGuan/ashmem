#include <linux/ashmem.h>
#include <fcntl.h>
#include <sys/ioctl.h>
#include <string.h>
#include <stdio.h>

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

void test(void) {
    printf("ASHMEM_SET_NAME: %lu\n"
           "ASHMEM_GET_NAME: %lu\n"
           "ASHMEM_SET_SIZE: %lu\n"
           "ASHMEM_GET_SIZE: %u\n"
           "ASHMEM_SET_PROT_MASK: %lu\n"
           "ASHMEM_GET_PROT_MASK: %u\n"
           "ASHMEM_PIN: %lu\n"
           "ASHMEM_UNPIN: %lu\n"
           "ASHMEM_GET_PIN_STATUS: %u\n"
           "ASHMEM_PURGE_ALL_CACHES: %u\n",
           ASHMEM_SET_NAME,
           ASHMEM_GET_NAME,
           ASHMEM_SET_SIZE,
           ASHMEM_GET_SIZE,
           ASHMEM_SET_PROT_MASK,
           ASHMEM_GET_PROT_MASK,
           ASHMEM_PIN,
           ASHMEM_UNPIN,
           ASHMEM_GET_PIN_STATUS,
           ASHMEM_PURGE_ALL_CACHES);
    printf("ulong: %lu, uint: %lu\n", sizeof(unsigned long), sizeof(unsigned int));
    fflush(stdout);
}
