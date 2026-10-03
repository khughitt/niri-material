// A real zwp_idle_inhibit_manager_v1 client for the optic-settling capture
// (material-80caf4): one mapped xdg_toplevel (app_id gos-inhibit) with a
// solid shm buffer, holding an idle inhibitor on its surface until SIGTERM,
// SIGINT or a close request. niri counts an inhibitor only while its surface
// has a primary scanout output, so the window must be mapped and drawn.
//
// optic-settling-smoke.sh builds it per run against the installed protocols:
//   wayland-scanner client-header / private-code for xdg-shell and
//   idle-inhibit-unstable-v1, then cc ... -lwayland-client.
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

#include "idle-inhibit-unstable-v1-client-protocol.h"
#include "xdg-shell-client-protocol.h"

enum { WIDTH = 240, HEIGHT = 160 };

static struct wl_compositor *compositor;
static struct wl_shm *shm;
static struct xdg_wm_base *wm_base;
static struct zwp_idle_inhibit_manager_v1 *inhibit_manager;
static struct wl_surface *surface;
static struct zwp_idle_inhibitor_v1 *inhibitor;
static volatile sig_atomic_t running = 1;

static void die(const char *what) {
    fprintf(stderr, "idle-inhibit-client: %s\n", what);
    exit(1);
}

static void on_signal(int sig) {
    (void)sig;
    running = 0;
}

static void registry_global(void *data, struct wl_registry *registry, uint32_t name,
                            const char *interface, uint32_t version) {
    (void)data;
    (void)version;
    if (strcmp(interface, wl_compositor_interface.name) == 0)
        compositor = wl_registry_bind(registry, name, &wl_compositor_interface, 4);
    else if (strcmp(interface, wl_shm_interface.name) == 0)
        shm = wl_registry_bind(registry, name, &wl_shm_interface, 1);
    else if (strcmp(interface, xdg_wm_base_interface.name) == 0)
        wm_base = wl_registry_bind(registry, name, &xdg_wm_base_interface, 1);
    else if (strcmp(interface, zwp_idle_inhibit_manager_v1_interface.name) == 0)
        inhibit_manager =
            wl_registry_bind(registry, name, &zwp_idle_inhibit_manager_v1_interface, 1);
}

static void registry_remove(void *data, struct wl_registry *registry, uint32_t name) {
    (void)data;
    (void)registry;
    (void)name;
}

static const struct wl_registry_listener registry_listener = {registry_global, registry_remove};

static void wm_base_ping(void *data, struct xdg_wm_base *base, uint32_t serial) {
    (void)data;
    xdg_wm_base_pong(base, serial);
}

static const struct xdg_wm_base_listener wm_base_listener = {wm_base_ping};

static struct wl_buffer *solid_buffer(void) {
    int stride = WIDTH * 4, size = stride * HEIGHT;
    int fd = memfd_create("idle-inhibit-client", MFD_CLOEXEC);
    if (fd < 0 || ftruncate(fd, size) < 0)
        die("shm file");
    uint32_t *pixels = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    if (pixels == MAP_FAILED)
        die("mmap");
    for (int i = 0; i < WIDTH * HEIGHT; i++)
        pixels[i] = 0xff404850;
    munmap(pixels, size);
    struct wl_shm_pool *pool = wl_shm_create_pool(shm, fd, size);
    struct wl_buffer *buffer =
        wl_shm_pool_create_buffer(pool, 0, WIDTH, HEIGHT, stride, WL_SHM_FORMAT_XRGB8888);
    wl_shm_pool_destroy(pool);
    close(fd);
    return buffer;
}

static void surface_configure(void *data, struct xdg_surface *xdg_surface, uint32_t serial) {
    (void)data;
    xdg_surface_ack_configure(xdg_surface, serial);
    static struct wl_buffer *buffer;
    if (!buffer) {
        buffer = solid_buffer();
        wl_surface_attach(surface, buffer, 0, 0);
        wl_surface_damage_buffer(surface, 0, 0, WIDTH, HEIGHT);
    }
    wl_surface_commit(surface);
}

static const struct xdg_surface_listener surface_listener = {surface_configure};

static void toplevel_configure(void *data, struct xdg_toplevel *toplevel, int32_t width,
                               int32_t height, struct wl_array *states) {
    (void)data;
    (void)toplevel;
    (void)width;
    (void)height;
    (void)states;
}

static void toplevel_close(void *data, struct xdg_toplevel *toplevel) {
    (void)data;
    (void)toplevel;
    running = 0;
}

// Bound at version 1: configure_bounds and wm_capabilities are never sent.
static const struct xdg_toplevel_listener toplevel_listener = {
    .configure = toplevel_configure,
    .close = toplevel_close,
};

int main(void) {
    struct sigaction action = {.sa_handler = on_signal};
    sigaction(SIGTERM, &action, NULL);
    sigaction(SIGINT, &action, NULL);

    struct wl_display *display = wl_display_connect(NULL);
    if (!display)
        die("cannot connect to the Wayland display");
    struct wl_registry *registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &registry_listener, NULL);
    wl_display_roundtrip(display);
    if (!compositor || !shm || !wm_base)
        die("missing wl_compositor, wl_shm or xdg_wm_base");
    if (!inhibit_manager)
        die("the compositor offers no zwp_idle_inhibit_manager_v1");
    xdg_wm_base_add_listener(wm_base, &wm_base_listener, NULL);

    surface = wl_compositor_create_surface(compositor);
    struct xdg_surface *xdg_surface = xdg_wm_base_get_xdg_surface(wm_base, surface);
    xdg_surface_add_listener(xdg_surface, &surface_listener, NULL);
    struct xdg_toplevel *toplevel = xdg_surface_get_toplevel(xdg_surface);
    xdg_toplevel_add_listener(toplevel, &toplevel_listener, NULL);
    xdg_toplevel_set_app_id(toplevel, "gos-inhibit");
    xdg_toplevel_set_title(toplevel, "idle inhibitor");
    inhibitor = zwp_idle_inhibit_manager_v1_create_inhibitor(inhibit_manager, surface);
    wl_surface_commit(surface);

    struct pollfd fd = {.fd = wl_display_get_fd(display), .events = POLLIN};
    while (running) {
        while (wl_display_prepare_read(display) != 0)
            wl_display_dispatch_pending(display);
        if (wl_display_flush(display) < 0 && errno != EAGAIN) {
            wl_display_cancel_read(display);
            die("flush failed");
        }
        if (poll(&fd, 1, -1) < 0) {
            wl_display_cancel_read(display);
            if (errno == EINTR)
                continue;
            die("poll failed");
        }
        if (wl_display_read_events(display) < 0)
            die("the compositor closed the connection");
        wl_display_dispatch_pending(display);
    }

    zwp_idle_inhibitor_v1_destroy(inhibitor);
    xdg_toplevel_destroy(toplevel);
    xdg_surface_destroy(xdg_surface);
    wl_surface_destroy(surface);
    wl_display_roundtrip(display);
    wl_display_disconnect(display);
    return 0;
}
