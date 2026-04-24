#include "include/listener/listener_plugin.h"

#include <flutter_linux/flutter_linux.h>
#include <gdk-pixbuf/gdk-pixbuf.h>
#include <gio/gio.h>
#include <gtk/gtk.h>
#include <sys/utsname.h>

#ifdef GDK_WINDOWING_X11
#include <gdk/gdkx.h>
#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/keysym.h>
#include <X11/extensions/XTest.h>
#endif

#include <ctype.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "listener_plugin_private.h"

// Clipboard content type codes — must match Dart ClipboardDataType enum order.
#define CLIP_TYPE_TEXT    0
#define CLIP_TYPE_IMAGE   1
#define CLIP_TYPE_FILE    2
#define CLIP_TYPE_FOLDER  3
#define CLIP_TYPE_LINK    4
#define CLIP_TYPE_AUDIO   5
#define CLIP_TYPE_VIDEO   6

#define LISTENER_PLUGIN(obj) \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), listener_plugin_get_type(), ListenerPlugin))

static const gchar* kClipboardChannelName = "copypaste/clipboard";
static const gchar* kClipboardWriterChannelName = "copypaste/clipboard_writer";
static const guint64 kClipboardDebounceMs = 500;
static const guint kClipboardPollIntervalMs = 250;
static const guint64 kClipboardWriteIgnoreMs = 700;

typedef struct {
#ifdef GDK_WINDOWING_X11
  Window window;
#else
  unsigned long window;
#endif
  gboolean valid;
} ActiveX11Window;

struct _ListenerPlugin {
  GObject parent_instance;

  FlEventChannel* event_channel;
  FlMethodChannel* method_channel;

  gboolean is_listening;
  guint poll_timer_id;
  gchar* last_content_hash;
  guint64 last_change_tick_ms;
  guint64 last_write_tick_ms;
};

G_DEFINE_TYPE(ListenerPlugin, listener_plugin, g_object_get_type())

static guint64 now_ms(void) {
  return (guint64)(g_get_monotonic_time() / 1000);
}

static gchar* compute_fnv1a_hash(const gchar* text) {
  uint64_t hash = 14695981039346656037ULL;
  const guchar* bytes = (const guchar*)text;
  for (gsize i = 0; bytes[i] != 0; i++) {
    hash ^= bytes[i];
    hash *= 1099511628211ULL;
  }
  return g_strdup_printf("%" G_GINT64_MODIFIER "x", (guint64)hash);
}

static gboolean is_url_text(const gchar* text) {
  if (text == NULL || *text == '\0') {
    return FALSE;
  }

  const gchar* prefixes[] = {
      "https://", "http://", "ftp://", "file:///", "mailto:", NULL,
  };

  gchar* lower = g_ascii_strdown(text, -1);
  gboolean matches = FALSE;
  for (guint i = 0; prefixes[i] != NULL; i++) {
    if (g_str_has_prefix(lower, prefixes[i])) {
      matches = TRUE;
      break;
    }
  }
  g_free(lower);

  return matches && strchr(text, ' ') == NULL && strchr(text, '\n') == NULL;
}

static int detect_file_type(const gchar* path) {
  if (path == NULL || *path == '\0') {
    return CLIP_TYPE_FILE;
  }

  if (g_file_test(path, G_FILE_TEST_IS_DIR)) {
    return CLIP_TYPE_FOLDER;
  }

  gchar* lower = g_ascii_strdown(path, -1);
  const gchar* ext = strrchr(lower, '.');
  int type = CLIP_TYPE_FILE;

  if (ext != NULL) {
    if (g_strcmp0(ext, ".png") == 0 || g_strcmp0(ext, ".jpg") == 0 ||
        g_strcmp0(ext, ".jpeg") == 0 || g_strcmp0(ext, ".gif") == 0 ||
        g_strcmp0(ext, ".bmp") == 0 || g_strcmp0(ext, ".webp") == 0 ||
        g_strcmp0(ext, ".svg") == 0 || g_strcmp0(ext, ".ico") == 0 ||
        g_strcmp0(ext, ".tiff") == 0 || g_strcmp0(ext, ".heic") == 0) {
      type = CLIP_TYPE_IMAGE;
    } else if (g_strcmp0(ext, ".mp3") == 0 || g_strcmp0(ext, ".wav") == 0 ||
               g_strcmp0(ext, ".flac") == 0 || g_strcmp0(ext, ".aac") == 0 ||
               g_strcmp0(ext, ".ogg") == 0 || g_strcmp0(ext, ".m4a") == 0) {
      type = CLIP_TYPE_AUDIO;
    } else if (g_strcmp0(ext, ".mp4") == 0 || g_strcmp0(ext, ".avi") == 0 ||
               g_strcmp0(ext, ".mkv") == 0 || g_strcmp0(ext, ".mov") == 0 ||
               g_strcmp0(ext, ".wmv") == 0 || g_strcmp0(ext, ".flv") == 0 ||
               g_strcmp0(ext, ".webm") == 0) {
      type = CLIP_TYPE_VIDEO;
    }
  }

  g_free(lower);
  return type;
}

static gboolean plugin_is_x11(void) {
#ifdef GDK_WINDOWING_X11
  GdkDisplay* display = gdk_display_get_default();
  return display != NULL && GDK_IS_X11_DISPLAY(display);
#else
  return FALSE;
#endif
}

#ifdef GDK_WINDOWING_X11
// Cached X11 atoms — interned once per process.
static Atom s_atom_net_active_window = None;
static Atom s_atom_net_wm_pid = None;

static Atom atom_net_active_window(Display* display) {
  if (s_atom_net_active_window == None) {
    s_atom_net_active_window = XInternAtom(display, "_NET_ACTIVE_WINDOW", False);
  }
  return s_atom_net_active_window;
}

static Atom atom_net_wm_pid(Display* display) {
  if (s_atom_net_wm_pid == None) {
    s_atom_net_wm_pid = XInternAtom(display, "_NET_WM_PID", False);
  }
  return s_atom_net_wm_pid;
}

// XTest extension availability — checked once per process.
static gboolean s_xtest_checked = FALSE;
static gboolean s_xtest_available = FALSE;

static gboolean ensure_xtest(Display* display) {
  if (s_xtest_checked) {
    return s_xtest_available;
  }
  s_xtest_checked = TRUE;
  int event_base, error_base, major, minor;
  s_xtest_available = XTestQueryExtension(display, &event_base, &error_base,
                                          &major, &minor) != 0;
  if (!s_xtest_available) {
    g_warning("XTest extension not available — paste simulation disabled");
  }
  return s_xtest_available;
}

static Display* get_xdisplay(void) {
  GdkDisplay* display = gdk_display_get_default();
  if (display == NULL || !GDK_IS_X11_DISPLAY(display)) {
    return NULL;
  }

  return gdk_x11_display_get_xdisplay(display);
}

static ActiveX11Window get_active_x11_window(void) {
  ActiveX11Window result = {0};
  Display* display = get_xdisplay();
  if (display == NULL) {
    return result;
  }

  Atom property = atom_net_active_window(display);
  Atom actual_type = None;
  int actual_format = 0;
  unsigned long item_count = 0;
  unsigned long bytes_after = 0;
  unsigned char* data = NULL;

  if (XGetWindowProperty(display, DefaultRootWindow(display), property, 0, 1,
                         False, AnyPropertyType, &actual_type, &actual_format,
                         &item_count, &bytes_after, &data) == Success &&
      data != NULL && item_count == 1) {
    result.window = *(Window*)data;
    result.valid = result.window != 0;
  }

  (void)actual_type;
  (void)actual_format;
  (void)bytes_after;

  if (data != NULL) {
    XFree(data);
  }

  return result;
}

static gchar* read_proc_comm(unsigned long pid) {
  gchar path[64];
  g_snprintf(path, sizeof(path), "/proc/%lu/comm", pid);
  gchar* content = NULL;
  gsize length = 0;
  if (!g_file_get_contents(path, &content, &length, NULL) || content == NULL) {
    return NULL;
  }

  g_strchomp(content);
  return content;
}

static gchar* get_x11_window_source(Window window) {
  Display* display = get_xdisplay();
  if (display == NULL || window == 0) {
    return g_strdup("");
  }

  XClassHint class_hint;
  if (XGetClassHint(display, window, &class_hint) != 0) {
    gchar* value = g_strdup(class_hint.res_class != NULL ? class_hint.res_class
                                                          : class_hint.res_name);
    if (class_hint.res_name != NULL) {
      XFree(class_hint.res_name);
    }
    if (class_hint.res_class != NULL) {
      XFree(class_hint.res_class);
    }
    if (value != NULL && *value != '\0') {
      return value;
    }
    g_free(value);
  }

  Atom pid_atom = atom_net_wm_pid(display);
  Atom actual_type = None;
  int actual_format = 0;
  unsigned long item_count = 0;
  unsigned long bytes_after = 0;
  unsigned char* data = NULL;

  if (XGetWindowProperty(display, window, pid_atom, 0, 1, False,
                         XA_CARDINAL, &actual_type, &actual_format,
                         &item_count, &bytes_after, &data) == Success &&
      data != NULL && item_count == 1) {
    unsigned long pid = *(unsigned long*)data;
    XFree(data);
    data = NULL;
    gchar* comm = read_proc_comm(pid);
    if (comm != NULL) {
      return comm;
    }
  }

  (void)actual_type;
  (void)actual_format;
  (void)bytes_after;

  if (data != NULL) {
    XFree(data);
  }

  return g_strdup("");
}

static gchar* capture_frontmost_x11_identifier(void) {
  ActiveX11Window active = get_active_x11_window();
  if (!active.valid) {
    return NULL;
  }

  return g_strdup_printf("x11:0x%lx", (unsigned long)active.window);
}

static int activate_noop_error_handler(Display* display, XErrorEvent* event) {
  (void)display;
  (void)event;
  return 0;
}

static gboolean request_activate_x11_window(Window window) {
  Display* display = get_xdisplay();
  if (display == NULL || window == 0) {
    return FALSE;
  }

  // 1. Send the EWMH _NET_ACTIVE_WINDOW message (honours ICCCM; most WMs).
  //    source=2 (pager) is more trusted than 1 (application) on WMs that
  //    apply focus-stealing prevention (KDE Plasma, some GNOME configs).
  XEvent event;
  memset(&event, 0, sizeof(event));
  event.xclient.type = ClientMessage;
  event.xclient.window = window;
  event.xclient.message_type = atom_net_active_window(display);
  event.xclient.format = 32;
  event.xclient.data.l[0] = 2;  // pager source — more likely to bypass focus-steal guards
  event.xclient.data.l[1] = CurrentTime;
  event.xclient.data.l[2] = 0;
  event.xclient.data.l[3] = 0;
  event.xclient.data.l[4] = 0;

  Status status = XSendEvent(display, DefaultRootWindow(display), False,
                             SubstructureNotifyMask | SubstructureRedirectMask,
                             &event);

  // 2. Raise the window and attempt a direct input focus as a fallback for WMs
  //    that ignore _NET_ACTIVE_WINDOW (tiling WMs, minimal WMs).
  //    Trap X errors: XSetInputFocus produces BadMatch on unmapped/invisible windows.
  XRaiseWindow(display, window);
  XSync(display, False);
  int (*prev_handler)(Display*, XErrorEvent*) = XSetErrorHandler(activate_noop_error_handler);
  XSetInputFocus(display, window, RevertToParent, CurrentTime);
  XSync(display, False);
  XSetErrorHandler(prev_handler);

  XFlush(display);
  return status != 0;
}

static gboolean simulate_paste_x11(void) {
  Display* display = get_xdisplay();
  if (display == NULL) {
    return FALSE;
  }

  if (!ensure_xtest(display)) {
    return FALSE;
  }

  KeyCode ctrl = XKeysymToKeycode(display, XK_Control_L);
  KeyCode v = XKeysymToKeycode(display, XK_v);
  if (ctrl == 0 || v == 0) {
    return FALSE;
  }

  XTestFakeKeyEvent(display, ctrl, True, CurrentTime);
  XTestFakeKeyEvent(display, v, True, CurrentTime);
  XTestFakeKeyEvent(display, v, False, CurrentTime);
  XTestFakeKeyEvent(display, ctrl, False, CurrentTime);
  XFlush(display);
  return TRUE;
}
#endif

static gchar* get_clipboard_source(void) {
#ifdef GDK_WINDOWING_X11
  if (plugin_is_x11()) {
    ActiveX11Window active = get_active_x11_window();
    if (active.valid) {
      return get_x11_window_source(active.window);
    }
  }
#endif
  return g_strdup("");
}

static GtkSelectionData* get_target_contents(GtkClipboard* clipboard,
                                             const gchar* target_name) {
  GdkAtom atom = gdk_atom_intern(target_name, FALSE);
  return gtk_clipboard_wait_for_contents(clipboard, atom);
}

static FlValue* get_selection_data_value(GtkClipboard* clipboard,
                                         const gchar* const* targets) {
  for (guint i = 0; targets[i] != NULL; i++) {
    GtkSelectionData* data = get_target_contents(clipboard, targets[i]);
    if (data == NULL) {
      continue;
    }

    gint length = gtk_selection_data_get_length(data);
    const guchar* bytes = gtk_selection_data_get_data(data);
    FlValue* result = NULL;
    if (bytes != NULL && length > 0) {
      result = fl_value_new_uint8_list(bytes, (size_t)length);
    }

    gtk_selection_data_free(data);
    if (result != NULL) {
      return result;
    }
  }

  return NULL;
}

static gchar* build_clipboard_signature(GtkClipboard* clipboard) {
  GString* signature = g_string_new("");

  gchar** uris = gtk_clipboard_wait_for_uris(clipboard);
  if (uris != NULL && uris[0] != NULL) {
    for (guint i = 0; uris[i] != NULL; i++) {
      g_autofree gchar* path = g_filename_from_uri(uris[i], NULL, NULL);
      if (path != NULL) {
        g_string_append_printf(signature, "F:%s|", path);
      } else {
        g_string_append_printf(signature, "U:%s|", uris[i]);
      }
    }
    g_strfreev(uris);
    return g_string_free(signature, FALSE);
  }

  if (uris != NULL) {
    g_strfreev(uris);
  }

  gchar* text = gtk_clipboard_wait_for_text(clipboard);
  if (text != NULL && *text != '\0') {
    gsize length = strlen(text);
    gsize sample_length = length > 100 ? 100 : length;
    g_string_append(signature, "T:");
    g_string_append_len(signature, text, sample_length);
    g_free(text);
    return g_string_free(signature, FALSE);
  }
  g_free(text);

  GdkPixbuf* image = gtk_clipboard_wait_for_image(clipboard);
  if (image != NULL) {
    const guchar* pixels = gdk_pixbuf_read_pixels(image);
    gsize rowstride = (gsize)gdk_pixbuf_get_rowstride(image);
    gint height = gdk_pixbuf_get_height(image);
    gsize total = rowstride * (gsize)height;
    gsize sample_len = total > 256 ? 256 : total;
    g_string_append(signature, "I:");
    g_string_append_printf(signature, "%" G_GSIZE_FORMAT ":", total);
    for (gsize i = 0; i < sample_len; i++) {
      g_string_append_printf(signature, "%02x", pixels[i]);
    }
    g_object_unref(image);
    return g_string_free(signature, FALSE);
  }

  return g_string_free(signature, FALSE);
}

static gboolean is_duplicate_change(ListenerPlugin* self, const gchar* hash) {
  guint64 now = now_ms();
  if (self->last_content_hash != NULL && g_strcmp0(self->last_content_hash, hash) == 0 &&
      (now - self->last_change_tick_ms) < kClipboardDebounceMs) {
    return TRUE;
  }

  g_free(self->last_content_hash);
  self->last_content_hash = g_strdup(hash);
  self->last_change_tick_ms = now;
  return FALSE;
}

static gboolean should_ignore_recent_write(ListenerPlugin* self) {
  guint64 now = now_ms();
  return self->last_write_tick_ms != 0 &&
         (now - self->last_write_tick_ms) < kClipboardWriteIgnoreMs;
}

static gboolean send_clipboard_event(ListenerPlugin* self, FlValue* event) {
  if (!self->is_listening || self->event_channel == NULL || event == NULL) {
    return FALSE;
  }

  g_autoptr(GError) error = NULL;
  gboolean success = fl_event_channel_send(self->event_channel, event, NULL, &error);
  if (!success && error != NULL) {
    g_warning("Failed to send clipboard event: %s", error->message);
  }
  return success;
}

static FlValue* build_file_event(GtkClipboard* clipboard,
                                 const gchar* source,
                                 const gchar* hash) {
  gchar** uris = gtk_clipboard_wait_for_uris(clipboard);
  if (uris == NULL || uris[0] == NULL) {
    g_strfreev(uris);
    return NULL;
  }

  g_autoptr(FlValue) files = fl_value_new_list();
  guint count = 0;
  gint event_type = CLIP_TYPE_FILE;
  gchar* first_path = NULL;

  for (guint i = 0; uris[i] != NULL; i++) {
    g_autofree gchar* path = g_filename_from_uri(uris[i], NULL, NULL);
    if (path == NULL || *path == '\0') {
      continue;
    }
    if (first_path == NULL) {
      first_path = g_strdup(path);
    }
    fl_value_append_take(files, fl_value_new_string(path));
    count++;
  }

  g_strfreev(uris);

  if (count == 0) {
    g_free(first_path);
    return NULL;
  }

  if (count == 1 && first_path != NULL) {
    event_type = detect_file_type(first_path);
  }
  g_free(first_path);

  g_autoptr(FlValue) event = fl_value_new_map();
  fl_value_set_string_take(event, "type", fl_value_new_int(event_type));
  fl_value_set_string_take(event, "files", fl_value_ref(files));
  fl_value_set_string_take(event, "source", fl_value_new_string(source));
  fl_value_set_string_take(event, "contentHash", fl_value_new_string(hash));
  return fl_value_ref(event);
}

static FlValue* build_text_event(GtkClipboard* clipboard,
                                 const gchar* source,
                                 const gchar* hash) {
  gchar* text = gtk_clipboard_wait_for_text(clipboard);
  if (text == NULL || *text == '\0') {
    g_free(text);
    return NULL;
  }

  g_autoptr(FlValue) event = fl_value_new_map();
  fl_value_set_string_take(event, "type",
                           fl_value_new_int(is_url_text(text) ? CLIP_TYPE_LINK : CLIP_TYPE_TEXT));
  fl_value_set_string_take(event, "text", fl_value_new_string(text));
  fl_value_set_string_take(event, "source", fl_value_new_string(source));
  fl_value_set_string_take(event, "contentHash", fl_value_new_string(hash));

  const gchar* const rtf_targets[] = {"text/rtf", "application/rtf",
                                      "Rich Text Format", NULL};
  const gchar* const html_targets[] = {"text/html", "HTML Format", NULL};

  FlValue* rtf = get_selection_data_value(clipboard, rtf_targets);
  if (rtf != NULL) {
    fl_value_set_string_take(event, "rtf", rtf);
  }
  FlValue* html = get_selection_data_value(clipboard, html_targets);
  if (html != NULL) {
    fl_value_set_string_take(event, "html", html);
  }

  g_free(text);
  return fl_value_ref(event);
}

static FlValue* build_image_event(GtkClipboard* clipboard,
                                  const gchar* source,
                                  const gchar* hash) {
  GdkPixbuf* pixbuf = gtk_clipboard_wait_for_image(clipboard);
  if (pixbuf == NULL) {
    return NULL;
  }

  gchar* buffer = NULL;
  gsize buffer_size = 0;
  g_autoptr(GError) error = NULL;
  gboolean ok = gdk_pixbuf_save_to_buffer(pixbuf, &buffer, &buffer_size, "png",
                                          &error, NULL);
  g_object_unref(pixbuf);
  if (!ok || buffer == NULL || buffer_size == 0) {
    if (error != NULL) {
      g_warning("Failed to serialize clipboard image: %s", error->message);
    }
    g_free(buffer);
    return NULL;
  }

  g_autoptr(FlValue) event = fl_value_new_map();
  fl_value_set_string_take(event, "type", fl_value_new_int(CLIP_TYPE_IMAGE));
  fl_value_set_string_take(event, "bytes",
                           fl_value_new_uint8_list((const uint8_t*)buffer,
                                                   (size_t)buffer_size));
  fl_value_set_string_take(event, "source", fl_value_new_string(source));
  fl_value_set_string_take(event, "contentHash", fl_value_new_string(hash));

  g_free(buffer);
  return fl_value_ref(event);
}

static void process_clipboard(ListenerPlugin* self) {
  GtkClipboard* clipboard = gtk_clipboard_get(GDK_SELECTION_CLIPBOARD);
  if (clipboard == NULL) {
    return;
  }

  if (should_ignore_recent_write(self)) {
    return;
  }

  g_autofree gchar* signature = build_clipboard_signature(clipboard);
  if (signature == NULL || *signature == '\0') {
    return;
  }

  g_autofree gchar* hash = compute_fnv1a_hash(signature);
  if (hash == NULL || *hash == '\0' || is_duplicate_change(self, hash)) {
    return;
  }

  g_autofree gchar* source = get_clipboard_source();

  g_autoptr(FlValue) event = build_file_event(clipboard, source, hash);
  if (event == NULL) {
    event = build_text_event(clipboard, source, hash);
  }
  if (event == NULL) {
    event = build_image_event(clipboard, source, hash);
  }

  if (event != NULL) {
    send_clipboard_event(self, event);
  }
}

static gboolean clipboard_poll_cb(gpointer user_data) {
  ListenerPlugin* self = LISTENER_PLUGIN(user_data);
  if (!self->is_listening) {
    self->poll_timer_id = 0;
    return G_SOURCE_REMOVE;
  }

  process_clipboard(self);
  return G_SOURCE_CONTINUE;
}

static void ensure_polling(ListenerPlugin* self) {
  if (self->poll_timer_id == 0) {
    self->poll_timer_id = g_timeout_add(kClipboardPollIntervalMs,
                                        clipboard_poll_cb, self);
  }
}

static void stop_polling(ListenerPlugin* self) {
  if (self->poll_timer_id != 0) {
    g_source_remove(self->poll_timer_id);
    self->poll_timer_id = 0;
  }
}

static FlValue* get_cursor_and_screen_info(void) {
  GdkDisplay* display = gdk_display_get_default();
  if (display == NULL) {
    return NULL;
  }

  GdkSeat* seat = gdk_display_get_default_seat(display);
  if (seat == NULL) {
    return NULL;
  }

  GdkDevice* pointer = gdk_seat_get_pointer(seat);
  if (pointer == NULL) {
    return NULL;
  }

  gint cursor_x = 0;
  gint cursor_y = 0;
  gdk_device_get_position(pointer, NULL, &cursor_x, &cursor_y);

  GdkMonitor* monitor = gdk_display_get_monitor_at_point(display, cursor_x, cursor_y);
  if (monitor == NULL) {
    return NULL;
  }

  GdkRectangle workarea;
  memset(&workarea, 0, sizeof(workarea));
  gdk_monitor_get_workarea(monitor, &workarea);

  g_autoptr(FlValue) info = fl_value_new_map();
  fl_value_set_string_take(info, "cursorX", fl_value_new_float((double)cursor_x));
  fl_value_set_string_take(info, "cursorY", fl_value_new_float((double)cursor_y));
  fl_value_set_string_take(info, "waLeft", fl_value_new_float((double)workarea.x));
  fl_value_set_string_take(info, "waTop", fl_value_new_float((double)workarea.y));
  fl_value_set_string_take(info, "waRight",
                           fl_value_new_float((double)(workarea.x + workarea.width)));
  fl_value_set_string_take(info, "waBottom",
                           fl_value_new_float((double)(workarea.y + workarea.height)));
  return fl_value_ref(info);
}

static gboolean set_text_to_clipboard(const gchar* text) {
  if (text == NULL || *text == '\0') {
    return FALSE;
  }

  GtkClipboard* clipboard = gtk_clipboard_get(GDK_SELECTION_CLIPBOARD);
  if (clipboard == NULL) {
    return FALSE;
  }

  gtk_clipboard_set_text(clipboard, text, -1);
  gtk_clipboard_store(clipboard);
  return TRUE;
}

typedef struct {
  GdkPixbuf* pixbuf;
  gchar* uri;
} ImageClipData;

static void image_clip_get_cb(GtkClipboard* clipboard,
                               GtkSelectionData* selection_data,
                               guint info,
                               gpointer user_data) {
  (void)clipboard;
  ImageClipData* d = (ImageClipData*)user_data;

  if (info == 0) {
    GdkAtom target = gdk_atom_intern_static_string("text/uri-list");
    gtk_selection_data_set(selection_data, target, 8,
                          (const guchar*)d->uri, (gint)strlen(d->uri));
  } else {
    gtk_selection_data_set_pixbuf(selection_data, d->pixbuf);
  }
}

static void image_clip_clear_cb(GtkClipboard* clipboard, gpointer user_data) {
  (void)clipboard;
  ImageClipData* d = (ImageClipData*)user_data;
  if (d->pixbuf) g_object_unref(d->pixbuf);
  g_free(d->uri);
  g_free(d);
}

static gboolean set_image_to_clipboard(const gchar* image_path) {
  if (image_path == NULL || *image_path == '\0') {
    return FALSE;
  }

  g_autoptr(GError) error = NULL;
  GdkPixbuf* pixbuf = gdk_pixbuf_new_from_file(image_path, &error);
  if (pixbuf == NULL) {
    if (error != NULL) {
      g_warning("Failed to load image for clipboard: %s", error->message);
    }
    return FALSE;
  }

  GtkClipboard* clipboard = gtk_clipboard_get(GDK_SELECTION_CLIPBOARD);
  if (clipboard == NULL) {
    g_object_unref(pixbuf);
    return FALSE;
  }

  GtkTargetList* tl = gtk_target_list_new(NULL, 0);
  gtk_target_list_add(tl, gdk_atom_intern_static_string("text/uri-list"), 0, 0);
  gtk_target_list_add_image_targets(tl, 1, TRUE);

  gint n_targets = 0;
  GtkTargetEntry* targets = gtk_target_table_new_from_list(tl, &n_targets);
  gtk_target_list_unref(tl);

  gchar* uri = g_filename_to_uri(image_path, NULL, NULL);
  if (uri == NULL) {
    g_object_unref(pixbuf);
    gtk_target_table_free(targets, n_targets);
    return FALSE;
  }

  gchar* uri_line = g_strdup_printf("%s\r\n", uri);
  g_free(uri);

  ImageClipData* data = g_new0(ImageClipData, 1);
  data->pixbuf = pixbuf;
  data->uri = uri_line;

  gboolean ok = gtk_clipboard_set_with_data(
      clipboard, targets, n_targets,
      image_clip_get_cb, image_clip_clear_cb, data);
  gtk_target_table_free(targets, n_targets);

  if (!ok) {
    g_object_unref(pixbuf);
    g_free(uri_line);
    g_free(data);
    return FALSE;
  }

  gtk_clipboard_store(clipboard);
  return TRUE;
}

static void clipboard_uri_list_get_cb(GtkClipboard* clipboard,
                                      GtkSelectionData* selection_data,
                                      guint info,
                                      gpointer user_data) {
  (void)clipboard;
  (void)info;

  const gchar* uri_list = (const gchar*)user_data;
  if (uri_list == NULL || *uri_list == '\0') {
    return;
  }

  GdkAtom target = gdk_atom_intern_static_string("text/uri-list");
  gtk_selection_data_set(selection_data, target, 8, (const guchar*)uri_list,
                         (gint)strlen(uri_list));
}

static void clipboard_uri_list_clear_cb(GtkClipboard* clipboard, gpointer user_data) {
  (void)clipboard;
  g_free(user_data);
}

static gboolean set_files_to_clipboard(const gchar* content) {
  if (content == NULL || *content == '\0') {
    return FALSE;
  }

  gchar** parts = g_strsplit(content, "\n", -1);
  g_autoptr(GString) uri_list = g_string_new(NULL);
  for (guint i = 0; parts[i] != NULL; i++) {
    if (parts[i][0] == '\0' || !g_file_test(parts[i], G_FILE_TEST_EXISTS)) {
      continue;
    }
    gchar* uri = g_filename_to_uri(parts[i], NULL, NULL);
    if (uri != NULL) {
      g_string_append(uri_list, uri);
      g_string_append(uri_list, "\r\n");
      g_free(uri);
    }
  }
  g_strfreev(parts);

  if (uri_list->len == 0) {
    return FALSE;
  }

  GtkClipboard* clipboard = gtk_clipboard_get(GDK_SELECTION_CLIPBOARD);
  if (clipboard == NULL) {
    return FALSE;
  }

  static GtkTargetEntry targets[] = {
      {(gchar*)"text/uri-list", 0, 0},
  };

  gchar* uri_payload = g_string_free(g_steal_pointer(&uri_list), FALSE);
  gboolean set_ok = gtk_clipboard_set_with_data(
      clipboard, targets, G_N_ELEMENTS(targets), clipboard_uri_list_get_cb,
      clipboard_uri_list_clear_cb, uri_payload);
  if (!set_ok) {
    g_free(uri_payload);
    return FALSE;
  }

  gtk_clipboard_store(clipboard);
  return TRUE;
}

static FlValue* get_media_info(void) {
  return NULL;
}

static void respond_success(FlMethodCall* method_call, FlValue* result) {
  g_autoptr(GError) error = NULL;
  if (!fl_method_call_respond_success(method_call, result, &error) && error != NULL) {
    g_warning("Failed to respond to method call: %s", error->message);
  }
}

#ifdef GDK_WINDOWING_X11
static gboolean paste_after_delay_cb(gpointer data) {
  FlMethodCall* mc = FL_METHOD_CALL(data);
  simulate_paste_x11();
  respond_success(mc, fl_value_new_bool(TRUE));
  g_object_unref(mc);
  return G_SOURCE_REMOVE;
}
#endif

static void listener_plugin_handle_method_call(ListenerPlugin* self,
                                               FlMethodCall* method_call) {
  const gchar* method = fl_method_call_get_name(method_call);
  FlValue* args = fl_method_call_get_args(method_call);

  if (strcmp(method, "getCapabilities") == 0) {
    g_autoptr(FlValue) caps = fl_value_new_map();
    fl_value_set_string_take(caps, "isX11", fl_value_new_bool(plugin_is_x11()));
#ifdef GDK_WINDOWING_X11
    Display* display = get_xdisplay();
    gboolean has_xtest = display != NULL && ensure_xtest(display);
#else
    gboolean has_xtest = FALSE;
#endif
    fl_value_set_string_take(caps, "hasXTest", fl_value_new_bool(has_xtest));
    respond_success(method_call, fl_value_ref(caps));
    return;
  }

  if (strcmp(method, "setClipboardContent") == 0) {
    FlValue* type_value = args != NULL ? fl_value_lookup_string(args, "type") : NULL;
    gint64 type = type_value != NULL ? fl_value_get_int(type_value) : -1;
    FlValue* content_value = args != NULL ? fl_value_lookup_string(args, "content") : NULL;
    const gchar* content = content_value != NULL &&
                                   fl_value_get_type(content_value) == FL_VALUE_TYPE_STRING
                               ? fl_value_get_string(content_value)
                               : "";
    gboolean success = FALSE;

    switch (type) {
      case CLIP_TYPE_TEXT:
      case CLIP_TYPE_LINK:
        success = set_text_to_clipboard(content);
        break;
      case CLIP_TYPE_IMAGE:
        success = set_image_to_clipboard(content);
        break;
      case CLIP_TYPE_FILE:
      case CLIP_TYPE_FOLDER:
      case CLIP_TYPE_AUDIO:
      case CLIP_TYPE_VIDEO:
        success = set_files_to_clipboard(content);
        break;
      default:
        success = FALSE;
        break;
    }

    if (success) {
      self->last_write_tick_ms = now_ms();
    }
    respond_success(method_call, fl_value_new_bool(success));
    return;
  }

  if (strcmp(method, "getMediaInfo") == 0) {
    respond_success(method_call, get_media_info());
    return;
  }

  if (strcmp(method, "captureFrontmostApp") == 0) {
#ifdef GDK_WINDOWING_X11
    if (plugin_is_x11()) {
      gchar* id = capture_frontmost_x11_identifier();
      FlValue* value = id != NULL ? fl_value_new_string(id) : NULL;
      g_free(id);
      respond_success(method_call, value);
      return;
    }
#endif
    respond_success(method_call, NULL);
    return;
  }

  if (strcmp(method, "activateAndPaste") == 0) {
#ifdef GDK_WINDOWING_X11
    if (plugin_is_x11()) {
      FlValue* id_value = args != NULL ? fl_value_lookup_string(args, "bundleId") : NULL;
      FlValue* delay_value = args != NULL ? fl_value_lookup_string(args, "delayMs") : NULL;
      const gchar* identifier = id_value != NULL &&
                                        fl_value_get_type(id_value) == FL_VALUE_TYPE_STRING
                                    ? fl_value_get_string(id_value)
                                    : NULL;
      gint64 delay_ms = delay_value != NULL ? fl_value_get_int(delay_value) : 0;
      gboolean activated = FALSE;

      if (identifier != NULL && g_str_has_prefix(identifier, "x11:0x")) {
        Window window = (Window)g_ascii_strtoull(identifier + 6, NULL, 16);
        activated = request_activate_x11_window(window);
      }

      if (activated && delay_ms > 0) {
        FlMethodCall* held_call = FL_METHOD_CALL(g_object_ref(method_call));
        guint timer_id = g_timeout_add((guint)delay_ms, paste_after_delay_cb, held_call);
        if (timer_id != 0) {
          return;  // held_call will be released by paste_after_delay_cb
        }
        // Timer registration failed — release the ref and fall through to immediate paste.
        g_object_unref(held_call);
        g_warning("activateAndPaste: g_timeout_add failed, pasting immediately");
      }

      if (activated) {
        simulate_paste_x11();
      }
      respond_success(method_call, fl_value_new_bool(activated));
      return;
    }
#endif
    respond_success(method_call, fl_value_new_bool(FALSE));
    return;
  }

  if (strcmp(method, "getCursorAndScreenInfo") == 0) {
    respond_success(method_call, get_cursor_and_screen_info());
    return;
  }

  if (strcmp(method, "checkAccessibility") == 0 ||
      strcmp(method, "requestAccessibility") == 0 ||
      strcmp(method, "openAccessibilitySettings") == 0) {
    respond_success(method_call, fl_value_new_bool(TRUE));
    return;
  }

  g_autoptr(FlMethodResponse) response =
      FL_METHOD_RESPONSE(fl_method_not_implemented_response_new());
  fl_method_call_respond(method_call, response, NULL);
}

static FlMethodErrorResponse* stream_listen_cb(FlEventChannel* channel,
                                               FlValue* args,
                                               gpointer user_data) {
  (void)channel;
  (void)args;
  ListenerPlugin* self = LISTENER_PLUGIN(user_data);
  self->is_listening = TRUE;
  ensure_polling(self);
  return NULL;
}

static FlMethodErrorResponse* stream_cancel_cb(FlEventChannel* channel,
                                               FlValue* args,
                                               gpointer user_data) {
  (void)channel;
  (void)args;
  ListenerPlugin* self = LISTENER_PLUGIN(user_data);
  self->is_listening = FALSE;
  stop_polling(self);
  return NULL;
}

static void method_call_cb(FlMethodChannel* channel,
                           FlMethodCall* method_call,
                           gpointer user_data) {
  (void)channel;
  listener_plugin_handle_method_call(LISTENER_PLUGIN(user_data), method_call);
}

FlMethodResponse* get_platform_version(void) {
  struct utsname uname_data = {};
  uname(&uname_data);
  g_autofree gchar* version = g_strdup_printf("Linux %s", uname_data.version);
  g_autoptr(FlValue) result = fl_value_new_string(version);
  return FL_METHOD_RESPONSE(fl_method_success_response_new(result));
}

static void listener_plugin_dispose(GObject* object) {
  ListenerPlugin* self = LISTENER_PLUGIN(object);
  stop_polling(self);
  self->is_listening = FALSE;

  g_clear_object(&self->event_channel);
  g_clear_object(&self->method_channel);
  g_free(self->last_content_hash);
  self->last_content_hash = NULL;

  G_OBJECT_CLASS(listener_plugin_parent_class)->dispose(object);
}

static void listener_plugin_class_init(ListenerPluginClass* klass) {
  G_OBJECT_CLASS(klass)->dispose = listener_plugin_dispose;
}

static void listener_plugin_init(ListenerPlugin* self) {
  self->last_content_hash = NULL;
  self->last_change_tick_ms = 0;
  self->last_write_tick_ms = 0;
  self->is_listening = FALSE;
  self->poll_timer_id = 0;
}

void listener_plugin_register_with_registrar(FlPluginRegistrar* registrar) {
  ListenerPlugin* plugin = LISTENER_PLUGIN(
      g_object_new(listener_plugin_get_type(), NULL));

  FlBinaryMessenger* messenger = fl_plugin_registrar_get_messenger(registrar);
  g_autoptr(FlStandardMethodCodec) codec = fl_standard_method_codec_new();

  plugin->event_channel = fl_event_channel_new(messenger, kClipboardChannelName,
                                               FL_METHOD_CODEC(codec));
  fl_event_channel_set_stream_handlers(plugin->event_channel, stream_listen_cb,
                                       stream_cancel_cb, g_object_ref(plugin),
                                       g_object_unref);

  plugin->method_channel = fl_method_channel_new(
      messenger, kClipboardWriterChannelName, FL_METHOD_CODEC(codec));
  fl_method_channel_set_method_call_handler(plugin->method_channel,
                                            method_call_cb,
                                            g_object_ref(plugin),
                                            g_object_unref);

  g_object_unref(plugin);
}
