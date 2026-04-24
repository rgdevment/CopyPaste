#include "copypaste_linux_shell.h"

#include <flutter_linux/flutter_linux.h>
#include <gio/gio.h>
#include <gtk/gtk.h>
#ifdef HAVE_APPINDICATOR
#include <libayatana-appindicator/app-indicator.h>
#endif
#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#ifdef GDK_WINDOWING_X11
#include <gdk/gdkx.h>
#include <X11/Xlib.h>
#include <X11/Xatom.h>
#include <X11/Xutil.h>
#include <X11/keysym.h>
#include <X11/extensions/XTest.h>
#endif

struct _CopyPasteLinuxShell {
  FlMethodChannel* method_channel;
  FlEventChannel* event_channel;
  gboolean events_listening;

#ifdef HAVE_APPINDICATOR
  AppIndicator* app_indicator;
#else
  GtkStatusIcon* tray_icon;
#endif
  GtkWidget* tray_menu;
  GtkWidget* toggle_item;
  GtkWidget* exit_item;
  gchar* resolved_icon_path;

  GtkWindow* gtk_window;

  gboolean hotkey_registered;
#ifdef GDK_WINDOWING_X11
  Display* xdisplay;
  Window root_window;
  guint hotkey_keycode;
  guint hotkey_modifiers;
  guint32 last_hotkey_time;
#endif
};

static const gchar* kShellChannelName = "copypaste/linux_shell";
static const gchar* kShellEventChannelName = "copypaste/linux_shell/events";

static gboolean shell_is_x11(void) {
#ifdef GDK_WINDOWING_X11
  GdkDisplay* display = gdk_display_get_default();
  return display != NULL && GDK_IS_X11_DISPLAY(display);
#else
  return FALSE;
#endif
}

static FlValue* shell_event(const gchar* type) {
  g_autoptr(FlValue) event = fl_value_new_map();
  fl_value_set_string_take(event, "type", fl_value_new_string(type));
  return fl_value_ref(event);
}

static void send_shell_event(CopyPasteLinuxShell* shell, const gchar* type) {
  if (!shell->events_listening || shell->event_channel == NULL) {
    return;
  }

  g_autoptr(FlValue) event = shell_event(type);
  g_autoptr(GError) error = NULL;
  if (!fl_event_channel_send(shell->event_channel, event, NULL, &error) &&
      error != NULL) {
    g_warning("Failed to send linux shell event: %s", error->message);
  }
}

static gchar* resolve_asset_path(const gchar* asset_path) {
  if (asset_path == NULL || *asset_path == '\0') {
    return NULL;
  }

  if (g_path_is_absolute(asset_path) && g_file_test(asset_path, G_FILE_TEST_EXISTS)) {
    return g_strdup(asset_path);
  }

  gchar exe_path[PATH_MAX + 1];
  ssize_t length = readlink("/proc/self/exe", exe_path, PATH_MAX);
  if (length <= 0) {
    return g_file_test(asset_path, G_FILE_TEST_EXISTS) ? g_strdup(asset_path) : NULL;
  }

  exe_path[length] = '\0';
  g_autofree gchar* exe_dir = g_path_get_dirname(exe_path);
  g_autofree gchar* flutter_asset_path =
      g_build_filename(exe_dir, "data", "flutter_assets", asset_path, NULL);
  if (g_file_test(flutter_asset_path, G_FILE_TEST_EXISTS)) {
    return g_strdup(flutter_asset_path);
  }

  g_autofree gchar* sibling_asset_path = g_build_filename(exe_dir, asset_path, NULL);
  if (g_file_test(sibling_asset_path, G_FILE_TEST_EXISTS)) {
    return g_strdup(sibling_asset_path);
  }

  return NULL;
}

static void destroy_tray_menu(CopyPasteLinuxShell* shell) {
  if (shell->tray_menu != NULL) {
    gtk_widget_destroy(shell->tray_menu);
    shell->tray_menu = NULL;
    shell->toggle_item = NULL;
    shell->exit_item = NULL;
  }
}

static void tray_toggle_cb(GtkMenuItem* item, gpointer user_data) {
  (void)item;
  send_shell_event((CopyPasteLinuxShell*)user_data, "toggle");
}

static void tray_exit_cb(GtkMenuItem* item, gpointer user_data) {
  (void)item;
  send_shell_event((CopyPasteLinuxShell*)user_data, "exit");
}

#ifndef HAVE_APPINDICATOR
static void tray_activate_cb(GtkStatusIcon* status_icon, gpointer user_data) {
  (void)status_icon;
  send_shell_event((CopyPasteLinuxShell*)user_data, "toggle");
}

static void tray_popup_menu_cb(GtkStatusIcon* status_icon,
                               guint button,
                               guint activate_time,
                               gpointer user_data) {
  CopyPasteLinuxShell* shell = (CopyPasteLinuxShell*)user_data;
  if (shell->tray_menu == NULL) {
    return;
  }

  gtk_menu_popup(GTK_MENU(shell->tray_menu), NULL, NULL,
                 gtk_status_icon_position_menu, status_icon, button,
                 activate_time);
}
#endif

static void rebuild_tray_menu(CopyPasteLinuxShell* shell,
                              const gchar* show_hide_label,
                              const gchar* exit_label) {
  destroy_tray_menu(shell);

  shell->tray_menu = gtk_menu_new();
  shell->toggle_item = gtk_menu_item_new_with_label(show_hide_label);
  shell->exit_item = gtk_menu_item_new_with_label(exit_label);
  GtkWidget* separator = gtk_separator_menu_item_new();

  g_signal_connect(shell->toggle_item, "activate", G_CALLBACK(tray_toggle_cb),
                   shell);
  g_signal_connect(shell->exit_item, "activate", G_CALLBACK(tray_exit_cb), shell);

  gtk_menu_shell_append(GTK_MENU_SHELL(shell->tray_menu), shell->toggle_item);
  gtk_menu_shell_append(GTK_MENU_SHELL(shell->tray_menu), separator);
  gtk_menu_shell_append(GTK_MENU_SHELL(shell->tray_menu), shell->exit_item);
  gtk_widget_show_all(shell->tray_menu);
}

static void parse_tray_args(FlValue* args,
                            const gchar** out_icon_path,
                            const gchar** out_tooltip,
                            const gchar** out_show_hide,
                            const gchar** out_exit_label) {
  FlValue* icon_value = args != NULL ? fl_value_lookup_string(args, "iconPath") : NULL;
  FlValue* tooltip_value = args != NULL ? fl_value_lookup_string(args, "tooltip") : NULL;
  FlValue* toggle_value = args != NULL ? fl_value_lookup_string(args, "showHideLabel") : NULL;
  FlValue* exit_value = args != NULL ? fl_value_lookup_string(args, "exitLabel") : NULL;

  *out_icon_path =
      icon_value != NULL && fl_value_get_type(icon_value) == FL_VALUE_TYPE_STRING
          ? fl_value_get_string(icon_value)
          : NULL;
  *out_tooltip =
      tooltip_value != NULL && fl_value_get_type(tooltip_value) == FL_VALUE_TYPE_STRING
          ? fl_value_get_string(tooltip_value)
          : "CopyPaste";
  *out_show_hide =
      toggle_value != NULL && fl_value_get_type(toggle_value) == FL_VALUE_TYPE_STRING
          ? fl_value_get_string(toggle_value)
          : "Show/Hide";
  *out_exit_label =
      exit_value != NULL && fl_value_get_type(exit_value) == FL_VALUE_TYPE_STRING
          ? fl_value_get_string(exit_value)
          : "Exit";
}

static gboolean init_tray(CopyPasteLinuxShell* shell, FlValue* args) {
  const gchar* icon_path;
  const gchar* tooltip;
  const gchar* show_hide;
  const gchar* exit_label;
  parse_tray_args(args, &icon_path, &tooltip, &show_hide, &exit_label);

  g_free(shell->resolved_icon_path);
  shell->resolved_icon_path = resolve_asset_path(icon_path);

#ifdef HAVE_APPINDICATOR
  if (shell->app_indicator == NULL) {
    shell->app_indicator = app_indicator_new(
        "com.rgdevment.copypaste", "copypaste",
        APP_INDICATOR_CATEGORY_APPLICATION_STATUS);
  }

  if (shell->resolved_icon_path != NULL) {
    g_autofree gchar* icon_dir =
        g_path_get_dirname(shell->resolved_icon_path);
    g_autofree gchar* icon_base =
        g_path_get_basename(shell->resolved_icon_path);
    gchar* dot = strrchr(icon_base, '.');
    if (dot != NULL) *dot = '\0';
    app_indicator_set_icon_theme_path(shell->app_indicator, icon_dir);
    app_indicator_set_icon_full(shell->app_indicator, icon_base, tooltip);
  }

  app_indicator_set_title(shell->app_indicator, tooltip);
  rebuild_tray_menu(shell, show_hide, exit_label);
  app_indicator_set_menu(shell->app_indicator, GTK_MENU(shell->tray_menu));
  app_indicator_set_status(shell->app_indicator, APP_INDICATOR_STATUS_ACTIVE);
#else
  if (shell->tray_icon == NULL) {
    shell->tray_icon = gtk_status_icon_new();
    g_signal_connect(shell->tray_icon, "activate",
                     G_CALLBACK(tray_activate_cb), shell);
    g_signal_connect(shell->tray_icon, "popup-menu",
                     G_CALLBACK(tray_popup_menu_cb), shell);
  }

  if (shell->resolved_icon_path != NULL) {
    gtk_status_icon_set_from_file(shell->tray_icon, shell->resolved_icon_path);
  }

  gtk_status_icon_set_tooltip_text(shell->tray_icon, tooltip);
  gtk_status_icon_set_visible(shell->tray_icon, TRUE);
  rebuild_tray_menu(shell, show_hide, exit_label);
#endif

  return TRUE;
}

static gboolean destroy_tray(CopyPasteLinuxShell* shell) {
  destroy_tray_menu(shell);

#ifdef HAVE_APPINDICATOR
  if (shell->app_indicator != NULL) {
    app_indicator_set_status(shell->app_indicator, APP_INDICATOR_STATUS_PASSIVE);
    g_clear_object(&shell->app_indicator);
  }
#else
  if (shell->tray_icon != NULL) {
    gtk_status_icon_set_visible(shell->tray_icon, FALSE);
    g_clear_object(&shell->tray_icon);
  }
#endif

  g_clear_pointer(&shell->resolved_icon_path, g_free);
  return TRUE;
}

static FlValue* make_hotkey_result(gboolean success, const char* error_code) {
  FlValue* map = fl_value_new_map();
  fl_value_set_string_take(map, "success", fl_value_new_bool(success));
  if (error_code != NULL) {
    fl_value_set_string_take(map, "errorCode", fl_value_new_string(error_code));
  }
  return map;
}

#ifdef GDK_WINDOWING_X11
static guint modifier_combinations[] = {0, LockMask, Mod2Mask, LockMask | Mod2Mask};
static int (*previous_x11_error_handler)(Display*, XErrorEvent*) = NULL;
static Display* trapped_x11_display = NULL;
static int trapped_x11_error_code = Success;

static int hotkey_x11_error_handler(Display* display, XErrorEvent* event) {
  if (display == trapped_x11_display) {
    trapped_x11_error_code = event->error_code;
    return 0;
  }

  if (previous_x11_error_handler != NULL) {
    return previous_x11_error_handler(display, event);
  }

  return 0;
}

static gboolean trap_x11_grab(Display* display,
                              Window root_window,
                              KeyCode keycode,
                              guint modifiers) {
  previous_x11_error_handler = XSetErrorHandler(hotkey_x11_error_handler);
  trapped_x11_display = display;
  trapped_x11_error_code = Success;

  XGrabKey(display, (int)keycode, (int)modifiers, root_window, True,
           GrabModeAsync, GrabModeAsync);
  XSync(display, False);

  trapped_x11_display = NULL;
  XSetErrorHandler(previous_x11_error_handler);
  previous_x11_error_handler = NULL;

  return trapped_x11_error_code == Success;
}

static void ungrab_hotkey_variants(Display* display,
                                   Window root_window,
                                   KeyCode keycode,
                                   guint modifiers) {
  for (guint i = 0; i < G_N_ELEMENTS(modifier_combinations); i++) {
    XUngrabKey(display, (int)keycode,
               (int)(modifiers | modifier_combinations[i]), root_window);
  }
  XSync(display, False);
}

static KeySym virtual_key_to_keysym(gint64 virtual_key) {
  if (virtual_key >= 0x41 && virtual_key <= 0x5A) {
    return (KeySym)(XK_A + (virtual_key - 0x41));
  }
  if (virtual_key >= 0x30 && virtual_key <= 0x39) {
    return (KeySym)(XK_0 + (virtual_key - 0x30));
  }
  if (virtual_key >= 0x70 && virtual_key <= 0x87) {
    return (KeySym)(XK_F1 + (virtual_key - 0x70));
  }
  switch (virtual_key) {
    case 0x08: return XK_BackSpace;
    case 0x09: return XK_Tab;
    case 0x0D: return XK_Return;
    case 0x1B: return XK_Escape;
    case 0x20: return XK_space;
    case 0x21: return XK_Page_Up;
    case 0x22: return XK_Page_Down;
    case 0x23: return XK_End;
    case 0x24: return XK_Home;
    case 0x25: return XK_Left;
    case 0x26: return XK_Up;
    case 0x27: return XK_Right;
    case 0x28: return XK_Down;
    case 0x2D: return XK_Insert;
    case 0x2E: return XK_Delete;
    case 0xBA: return XK_semicolon;
    case 0xBB: return XK_equal;
    case 0xBC: return XK_comma;
    case 0xBD: return XK_minus;
    case 0xBE: return XK_period;
    case 0xBF: return XK_slash;
    case 0xC0: return XK_grave;
    case 0xDB: return XK_bracketleft;
    case 0xDC: return XK_backslash;
    case 0xDD: return XK_bracketright;
    case 0xDE: return XK_apostrophe;
    default: return NoSymbol;
  }
}

static guint compute_modifier_mask(FlValue* args) {
  guint modifiers = 0;

  FlValue* ctrl = fl_value_lookup_string(args, "useCtrl");
  FlValue* meta = fl_value_lookup_string(args, "useWin");
  FlValue* alt = fl_value_lookup_string(args, "useAlt");
  FlValue* shift = fl_value_lookup_string(args, "useShift");

  if (ctrl != NULL && fl_value_get_type(ctrl) == FL_VALUE_TYPE_BOOL &&
      fl_value_get_bool(ctrl)) {
    modifiers |= ControlMask;
  }
  if (meta != NULL && fl_value_get_type(meta) == FL_VALUE_TYPE_BOOL &&
      fl_value_get_bool(meta)) {
    modifiers |= Mod4Mask;
  }
  if (alt != NULL && fl_value_get_type(alt) == FL_VALUE_TYPE_BOOL &&
      fl_value_get_bool(alt)) {
    modifiers |= Mod1Mask;
  }
  if (shift != NULL && fl_value_get_type(shift) == FL_VALUE_TYPE_BOOL &&
      fl_value_get_bool(shift)) {
    modifiers |= ShiftMask;
  }

  return modifiers;
}

static void unregister_hotkey(CopyPasteLinuxShell* shell) {
  if (!shell->hotkey_registered || shell->xdisplay == NULL || shell->hotkey_keycode == 0) {
    shell->hotkey_registered = FALSE;
    return;
  }

  ungrab_hotkey_variants(shell->xdisplay, shell->root_window,
                         (KeyCode)shell->hotkey_keycode,
                         shell->hotkey_modifiers);
  shell->hotkey_registered = FALSE;
  shell->hotkey_keycode = 0;
  shell->hotkey_modifiers = 0;
}

static FlValue* register_hotkey(CopyPasteLinuxShell* shell, FlValue* args) {
  if (!shell_is_x11() || shell->xdisplay == NULL) {
    return make_hotkey_result(FALSE, "noX11");
  }

  unregister_hotkey(shell);

  FlValue* key_value = args != NULL ? fl_value_lookup_string(args, "virtualKey") : NULL;
  gint64 virtual_key = (key_value != NULL && fl_value_get_type(key_value) == FL_VALUE_TYPE_INT)
      ? fl_value_get_int(key_value) : 0;
  KeySym keysym = virtual_key_to_keysym(virtual_key);
  if (keysym == NoSymbol) {
    g_warning("registerHotkey: unsupported virtual key 0x%llx", (unsigned long long)virtual_key);
    return make_hotkey_result(FALSE, "unsupportedKey");
  }

  guint modifiers = compute_modifier_mask(args);
  if (modifiers == 0) {
    g_warning("registerHotkey: no modifier keys specified");
    return make_hotkey_result(FALSE, "noModifier");
  }

  KeyCode keycode = XKeysymToKeycode(shell->xdisplay, keysym);
  if (keycode == 0) {
    g_warning("registerHotkey: no keycode for keysym %lu", (unsigned long)keysym);
    return make_hotkey_result(FALSE, "unsupportedKey");
  }

  for (guint i = 0; i < G_N_ELEMENTS(modifier_combinations); i++) {
    if (!trap_x11_grab(shell->xdisplay, shell->root_window, keycode,
                       modifiers | modifier_combinations[i])) {
      g_warning("registerHotkey: XGrabKey failed (modifier variant 0x%x) — key may be in use",
                modifiers | modifier_combinations[i]);
      ungrab_hotkey_variants(shell->xdisplay, shell->root_window, keycode,
                             modifiers);
      return make_hotkey_result(FALSE, "grabFailed");
    }
  }

  XSync(shell->xdisplay, False);
  XWindowAttributes attrs;
  if (XGetWindowAttributes(shell->xdisplay, shell->root_window, &attrs) != 0) {
    XSelectInput(shell->xdisplay, shell->root_window,
                 attrs.your_event_mask | KeyPressMask);
  } else {
    XSelectInput(shell->xdisplay, shell->root_window, KeyPressMask);
  }
  XSync(shell->xdisplay, False);

  shell->hotkey_registered = TRUE;
  shell->hotkey_keycode = keycode;
  shell->hotkey_modifiers = modifiers;
  return make_hotkey_result(TRUE, NULL);
}

static GdkFilterReturn x11_event_filter(GdkXEvent* xevent,
                                        GdkEvent* event,
                                        gpointer user_data) {
  (void)event;
  CopyPasteLinuxShell* shell = (CopyPasteLinuxShell*)user_data;
  if (!shell->hotkey_registered) {
    return GDK_FILTER_CONTINUE;
  }

  XEvent* x_event = (XEvent*)xevent;
  if (x_event->type != KeyPress) {
    return GDK_FILTER_CONTINUE;
  }

  guint relevant_mask = ControlMask | ShiftMask | Mod1Mask | Mod4Mask;
  guint state = (guint)x_event->xkey.state & relevant_mask;
  if ((guint)x_event->xkey.keycode == shell->hotkey_keycode &&
      state == shell->hotkey_modifiers) {
    shell->last_hotkey_time = x_event->xkey.time;
    send_shell_event(shell, "hotkey");
    return GDK_FILTER_REMOVE;
  }

  return GDK_FILTER_CONTINUE;
}
#endif

static FlMethodErrorResponse* shell_listen_cb(FlEventChannel* channel,
                                              FlValue* args,
                                              gpointer user_data) {
  (void)channel;
  (void)args;
  CopyPasteLinuxShell* shell = (CopyPasteLinuxShell*)user_data;
  shell->events_listening = TRUE;
  return NULL;
}

static FlMethodErrorResponse* shell_cancel_cb(FlEventChannel* channel,
                                              FlValue* args,
                                              gpointer user_data) {
  (void)channel;
  (void)args;
  CopyPasteLinuxShell* shell = (CopyPasteLinuxShell*)user_data;
  shell->events_listening = FALSE;
  return NULL;
}

static void respond_method_success(FlMethodCall* method_call, FlValue* result) {
  g_autoptr(GError) error = NULL;
  g_autoptr(FlValue) owned = result;
  if (!fl_method_call_respond_success(method_call, owned, &error) && error != NULL) {
    g_warning("Failed to respond to linux shell method call: %s", error->message);
  }
}

static gboolean has_app_indicator_runtime(void) {
#ifdef HAVE_APPINDICATOR
  return TRUE;
#else
  return FALSE;
#endif
}

static gboolean ewmh_supports_active_window(void) {
#ifdef GDK_WINDOWING_X11
  GdkDisplay* gdk_display = gdk_display_get_default();
  if (gdk_display == NULL || !GDK_IS_X11_DISPLAY(gdk_display)) {
    return FALSE;
  }
  Display* xdisplay = GDK_DISPLAY_XDISPLAY(gdk_display);
  if (xdisplay == NULL) return FALSE;
  Atom net_supported = XInternAtom(xdisplay, "_NET_SUPPORTED", True);
  Atom net_active_window = XInternAtom(xdisplay, "_NET_ACTIVE_WINDOW", True);
  if (net_supported == None || net_active_window == None) return FALSE;
  Window root = DefaultRootWindow(xdisplay);
  Atom actual_type = None;
  int actual_format = 0;
  unsigned long nitems = 0;
  unsigned long bytes_after = 0;
  unsigned char* data = NULL;
  int status = XGetWindowProperty(xdisplay, root, net_supported, 0, 1024, False,
                                  XA_ATOM, &actual_type, &actual_format,
                                  &nitems, &bytes_after, &data);
  gboolean found = FALSE;
  if (status == Success && actual_type == XA_ATOM && actual_format == 32 && data != NULL) {
    Atom* atoms = (Atom*)data;
    for (unsigned long i = 0; i < nitems; ++i) {
      if (atoms[i] == net_active_window) { found = TRUE; break; }
    }
  }
  if (data != NULL) XFree(data);
  return found;
#else
  return FALSE;
#endif
}

static gchar* read_wm_name(void) {
#ifdef GDK_WINDOWING_X11
  GdkDisplay* gdk_display = gdk_display_get_default();
  if (gdk_display == NULL || !GDK_IS_X11_DISPLAY(gdk_display)) return NULL;
  Display* xdisplay = GDK_DISPLAY_XDISPLAY(gdk_display);
  Atom check = XInternAtom(xdisplay, "_NET_SUPPORTING_WM_CHECK", True);
  Atom utf8 = XInternAtom(xdisplay, "UTF8_STRING", True);
  Atom wm_name = XInternAtom(xdisplay, "_NET_WM_NAME", True);
  if (check == None || wm_name == None) return NULL;
  Window root = DefaultRootWindow(xdisplay);
  Atom actual_type = None;
  int actual_format = 0;
  unsigned long nitems = 0;
  unsigned long bytes_after = 0;
  unsigned char* data = NULL;
  if (XGetWindowProperty(xdisplay, root, check, 0, 1, False, XA_WINDOW,
                         &actual_type, &actual_format, &nitems, &bytes_after,
                         &data) != Success || data == NULL) {
    return NULL;
  }
  Window wm_window = *(Window*)data;
  XFree(data);
  data = NULL;
  if (wm_window == None) return NULL;
  Atom string_type = utf8 != None ? utf8 : XA_STRING;
  if (XGetWindowProperty(xdisplay, wm_window, wm_name, 0, 256, False,
                         string_type, &actual_type, &actual_format, &nitems,
                         &bytes_after, &data) != Success || data == NULL) {
    return NULL;
  }
  gchar* name = g_strndup((const gchar*)data, nitems);
  XFree(data);
  return name;
#else
  return NULL;
#endif
}

static gboolean is_clipboard_manager_name(const gchar* comm) {
  if (comm == NULL) return FALSE;
  static const gchar* kKnown[] = {
    "klipper", "gpaste-applet", "gpaste-client", "clipman", "copyq",
    "xfce4-clipman", "parcellite", "diodon", "clipit", NULL,
  };
  for (int i = 0; kKnown[i] != NULL; ++i) {
    if (g_strcmp0(comm, kKnown[i]) == 0) return TRUE;
  }
  return FALSE;
}

static gboolean clipboard_manager_running(void) {
  GDir* dir = g_dir_open("/proc", 0, NULL);
  if (dir == NULL) return FALSE;
  gboolean found = FALSE;
  const gchar* name = NULL;
  while ((name = g_dir_read_name(dir)) != NULL) {
    gboolean only_digits = TRUE;
    for (const gchar* p = name; *p != '\0'; ++p) {
      if (!g_ascii_isdigit(*p)) { only_digits = FALSE; break; }
    }
    if (!only_digits) continue;
    g_autofree gchar* path = g_build_filename("/proc", name, "comm", NULL);
    g_autofree gchar* contents = NULL;
    gsize length = 0;
    if (!g_file_get_contents(path, &contents, &length, NULL)) continue;
    if (contents == NULL) continue;
    g_strstrip(contents);
    if (is_clipboard_manager_name(contents)) { found = TRUE; break; }
  }
  g_dir_close(dir);
  return found;
}

static FlValue* build_capabilities(void) {
  FlValue* caps = fl_value_new_map();
  fl_value_set_string_take(caps, "isX11", fl_value_new_bool(shell_is_x11()));
  fl_value_set_string_take(caps, "hasAppIndicator",
                           fl_value_new_bool(has_app_indicator_runtime()));
  fl_value_set_string_take(caps, "hasEwmh",
                           fl_value_new_bool(ewmh_supports_active_window()));
  fl_value_set_string_take(caps, "hasClipboardManager",
                           fl_value_new_bool(clipboard_manager_running()));
  const gchar* desktop_env = g_getenv("XDG_CURRENT_DESKTOP");
  if (desktop_env == NULL) desktop_env = g_getenv("DESKTOP_SESSION");
  fl_value_set_string_take(caps, "desktopEnv",
                           fl_value_new_string(desktop_env != NULL ? desktop_env : ""));
  g_autofree gchar* wm = read_wm_name();
  fl_value_set_string_take(caps, "wmName",
                           fl_value_new_string(wm != NULL ? wm : ""));
  return caps;
}

static void shell_method_call_cb(FlMethodChannel* channel,
                                 FlMethodCall* method_call,
                                 gpointer user_data) {
  (void)channel;
  CopyPasteLinuxShell* shell = (CopyPasteLinuxShell*)user_data;
  const gchar* method = fl_method_call_get_name(method_call);
  FlValue* args = fl_method_call_get_args(method_call);

  if (strcmp(method, "getCapabilities") == 0) {
    respond_method_success(method_call, build_capabilities());
    return;
  }

  if (strcmp(method, "initTray") == 0 || strcmp(method, "updateTray") == 0) {
    respond_method_success(method_call, fl_value_new_bool(init_tray(shell, args)));
    return;
  }

  if (strcmp(method, "destroyTray") == 0) {
    respond_method_success(method_call, fl_value_new_bool(destroy_tray(shell)));
    return;
  }

  if (strcmp(method, "registerHotkey") == 0) {
#ifdef GDK_WINDOWING_X11
    respond_method_success(method_call, register_hotkey(shell, args));
#else
    respond_method_success(method_call, make_hotkey_result(FALSE, "noX11"));
#endif
    return;
  }

  if (strcmp(method, "unregisterHotkey") == 0) {
#ifdef GDK_WINDOWING_X11
    unregister_hotkey(shell);
#endif
    respond_method_success(method_call, fl_value_new_bool(TRUE));
    return;
  }

  if (strcmp(method, "focusWindow") == 0) {
    if (shell->gtk_window != NULL) {
#ifdef GDK_WINDOWING_X11
      guint32 t = shell->last_hotkey_time != 0 ? shell->last_hotkey_time
                                                : GDK_CURRENT_TIME;
      gtk_window_present_with_time(shell->gtk_window, t);
#else
      gtk_window_present(shell->gtk_window);
#endif
    }
    respond_method_success(method_call, fl_value_new_bool(TRUE));
    return;
  }

  g_autoptr(FlMethodResponse) response =
      FL_METHOD_RESPONSE(fl_method_not_implemented_response_new());
  fl_method_call_respond(method_call, response, NULL);
}

CopyPasteLinuxShell* copypaste_linux_shell_new(FlBinaryMessenger* messenger,
                                               GtkWindow* window) {
  CopyPasteLinuxShell* shell = g_new0(CopyPasteLinuxShell, 1);
  shell->gtk_window = window;

  g_autoptr(FlStandardMethodCodec) codec = fl_standard_method_codec_new();
  shell->method_channel =
      fl_method_channel_new(messenger, kShellChannelName, FL_METHOD_CODEC(codec));
  fl_method_channel_set_method_call_handler(shell->method_channel,
                                            shell_method_call_cb, shell, NULL);

  shell->event_channel =
      fl_event_channel_new(messenger, kShellEventChannelName, FL_METHOD_CODEC(codec));
  fl_event_channel_set_stream_handlers(shell->event_channel, shell_listen_cb,
                                       shell_cancel_cb, shell, NULL);

#ifdef GDK_WINDOWING_X11
  if (shell_is_x11()) {
    GdkDisplay* display = gdk_display_get_default();
    shell->xdisplay = gdk_x11_display_get_xdisplay(display);
    shell->root_window = DefaultRootWindow(shell->xdisplay);
    gdk_window_add_filter(NULL, x11_event_filter, shell);
  }
#endif

  return shell;
}

void copypaste_linux_shell_dispose(CopyPasteLinuxShell* shell) {
  if (shell == NULL) {
    return;
  }

#ifdef GDK_WINDOWING_X11
  if (shell_is_x11()) {
    unregister_hotkey(shell);
    gdk_window_remove_filter(NULL, x11_event_filter, shell);
  }
#endif

  destroy_tray(shell);
  g_clear_object(&shell->method_channel);
  g_clear_object(&shell->event_channel);
  g_free(shell);
}
