#include <linux/ashmem.h>
#include <fcntl.h>
#include <sys/ioctl.h>

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
