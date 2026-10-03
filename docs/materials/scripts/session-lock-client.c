// session-lock-client.c: a PAM-free ext_session_lock_v1 client for the
// optic-settling TTY lane (docs/specs/2026-10-02-real-tty-settling-lane-design.md
// §4). Locks on start, gives every output a solid lock surface, prints
// "locked" when the compositor confirms, and on SIGUSR1 unlocks and exits 0.
// It never authenticates: niri's SessionLockHandler::unlock runs exactly as
// for any lock client. SIGTERM/SIGINT exit 2 without unlocking; a refused or
// ended lock ("finished") exits 1.
#define _GNU_SOURCE
#include <errno.h>
#include <poll.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include <wayland-client.h>

#include "ext-session-lock-v1-client-protocol.h"

enum { MAX_OUTPUTS = 8 };

struct lock_output {
    struct wl_output *output;
    struct wl_surface *surface;
    struct ext_session_lock_surface_v1 *lock_surface;
    struct wl_buffer *buffer;
};

static struct wl_compositor *compositor;
static struct wl_shm *shm;
static struct ext_session_lock_manager_v1 *manager;
static struct lock_output outputs[MAX_OUTPUTS];
static int n_outputs;
static int locked;
static volatile sig_atomic_t unlock_requested, stop_requested;

static void die(int status, const char *what) {
    fprintf(stderr, "session-lock-client: %s\n", what);
    exit(status);
}

static void on_usr1(int sig) {
    (void)sig;
    unlock_requested = 1;
}

static void on_stop(int sig) {
    (void)sig;
    stop_requested = 1;
}

static void registry_global(void *data, struct wl_registry *registry, uint32_t name,
                            const char *interface, uint32_t version) {
    (void)data;
    (void)version;
    if (strcmp(interface, wl_compositor_interface.name) == 0)
        compositor = wl_registry_bind(registry, name, &wl_compositor_interface, 4);
    else if (strcmp(interface, wl_shm_interface.name) == 0)
        shm = wl_registry_bind(registry, name, &wl_shm_interface, 1);
    else if (strcmp(interface, ext_session_lock_manager_v1_interface.name) == 0)
        manager = wl_registry_bind(registry, name, &ext_session_lock_manager_v1_interface, 1);
    else if (strcmp(interface, wl_output_interface.name) == 0 && n_outputs < MAX_OUTPUTS)
        outputs[n_outputs++].output = wl_registry_bind(registry, name, &wl_output_interface, 1);
}

static void registry_remove(void *data, struct wl_registry *registry, uint32_t name) {
    (void)data;
    (void)registry;
    (void)name;
}

static const struct wl_registry_listener registry_listener = {
    .global = registry_global,
    .global_remove = registry_remove,
};

static struct wl_buffer *solid_buffer(uint32_t width, uint32_t height) {
    int stride = (int)width * 4, size = stride * (int)height;
    int fd = memfd_create("session-lock-client", MFD_CLOEXEC);
    if (fd < 0 || ftruncate(fd, size) < 0)
        die(1, "shm file");
    uint32_t *pixels = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    if (pixels == MAP_FAILED)
        die(1, "mmap");
    for (uint32_t i = 0; i < width * height; i++)
        pixels[i] = 0xff202428;
    munmap(pixels, size);
    struct wl_shm_pool *pool = wl_shm_create_pool(shm, fd, size);
    struct wl_buffer *buffer = wl_shm_pool_create_buffer(pool, 0, (int)width, (int)height, stride,
                                                         WL_SHM_FORMAT_XRGB8888);
    wl_shm_pool_destroy(pool);
    close(fd);
    return buffer;
}

static void lock_surface_configure(void *data, struct ext_session_lock_surface_v1 *lock_surface,
                                   uint32_t serial, uint32_t width, uint32_t height) {
    struct lock_output *out = data;
    ext_session_lock_surface_v1_ack_configure(lock_surface, serial);
    if (out->buffer)
        wl_buffer_destroy(out->buffer);
    out->buffer = solid_buffer(width, height);
    wl_surface_attach(out->surface, out->buffer, 0, 0);
    wl_surface_damage_buffer(out->surface, 0, 0, (int32_t)width, (int32_t)height);
    wl_surface_commit(out->surface);
}

static const struct ext_session_lock_surface_v1_listener lock_surface_listener = {
    .configure = lock_surface_configure,
};

static void lock_locked(void *data, struct ext_session_lock_v1 *lock) {
    (void)data;
    (void)lock;
    locked = 1;
    printf("locked\n");
    fflush(stdout);
}

static void lock_finished(void *data, struct ext_session_lock_v1 *lock) {
    (void)data;
    (void)lock;
    die(1, "the compositor refused or ended the lock");
}

static const struct ext_session_lock_v1_listener lock_listener = {
    .locked = lock_locked,
    .finished = lock_finished,
};

int main(void) {
    // Block the signals so one arriving between the loop check and the wait
    // stays pending; ppoll unblocks them atomically while it waits.
    sigset_t blocked, orig_mask;
    sigemptyset(&blocked);
    sigaddset(&blocked, SIGUSR1);
    sigaddset(&blocked, SIGTERM);
    sigaddset(&blocked, SIGINT);
    sigprocmask(SIG_BLOCK, &blocked, &orig_mask);
    struct sigaction usr1 = {.sa_handler = on_usr1}, stop = {.sa_handler = on_stop};
    sigaction(SIGUSR1, &usr1, NULL);
    sigaction(SIGTERM, &stop, NULL);
    sigaction(SIGINT, &stop, NULL);

    struct wl_display *display = wl_display_connect(NULL);
    if (!display)
        die(1, "cannot connect to the Wayland display");
    struct wl_registry *registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &registry_listener, NULL);
    wl_display_roundtrip(display);
    if (!compositor || !shm || !manager || n_outputs == 0)
        die(1, "missing wl_compositor, wl_shm, ext_session_lock_manager_v1 or an output");

    struct ext_session_lock_v1 *lock = ext_session_lock_manager_v1_lock(manager);
    ext_session_lock_v1_add_listener(lock, &lock_listener, NULL);
    for (int i = 0; i < n_outputs; i++) {
        outputs[i].surface = wl_compositor_create_surface(compositor);
        outputs[i].lock_surface =
            ext_session_lock_v1_get_lock_surface(lock, outputs[i].surface, outputs[i].output);
        ext_session_lock_surface_v1_add_listener(outputs[i].lock_surface, &lock_surface_listener,
                                                 &outputs[i]);
    }

    struct pollfd fd = {.fd = wl_display_get_fd(display), .events = POLLIN};
    while (!unlock_requested && !stop_requested) {
        while (wl_display_prepare_read(display) != 0)
            wl_display_dispatch_pending(display);
        if (wl_display_flush(display) < 0 && errno != EAGAIN) {
            wl_display_cancel_read(display);
            die(1, "flush failed");
        }
        if (ppoll(&fd, 1, NULL, &orig_mask) < 0) {
            wl_display_cancel_read(display);
            if (errno == EINTR)
                continue;
            die(1, "poll failed");
        }
        if (wl_display_read_events(display) < 0)
            die(1, "the compositor closed the connection");
        wl_display_dispatch_pending(display);
    }
    if (stop_requested)
        die(2, "stopped without unlocking");
    if (!locked)
        die(1, "unlock requested before the compositor confirmed the lock");

    ext_session_lock_v1_unlock_and_destroy(lock);
    wl_display_roundtrip(display);
    wl_display_disconnect(display);
    return 0;
}
