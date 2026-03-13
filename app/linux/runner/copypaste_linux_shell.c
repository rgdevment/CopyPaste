#include "copypaste_linux_shell.h"

#include <flutter_linux/flutter_linux.h>
#include <gio/gio.h>
#include <gtk/gtk.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#ifdef GDK_WINDOWING_X11
#include <gdk/gdkx.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/keysym.h>
#include <X11/extensions/XTest.h>
#endif

struct _CopyPasteLinuxShell {
  FlMethodChannel* method_channel;
  FlEventChannel* event_channel;
  gboolean events_listening;

  GtkStatusIcon* tray_icon;
  GtkWidget* tray_menu;
  GtkWidget* toggle_item;
  GtkWidget* exit_item;
  gchar* resolved_icon_path;

  gboolean hotkey_registered;
#ifdef GDK_WINDOWING_X11
  Display* xdisplay;
  Window root_window;
  guint hotkey_keycode;
  guint hotkey_modifiers;
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

static gboolean init_tray(CopyPasteLinuxShell* shell, FlValue* args) {
  FlValue* icon_value = args != NULL ? fl_value_lookup_string(args, "iconPath") : NULL;
  FlValue* tooltip_value = args != NULL ? fl_value_lookup_string(args, "tooltip") : NULL;
  FlValue* toggle_value = args != NULL ? fl_value_lookup_string(args, "showHideLabel") : NULL;
  FlValue* exit_value = args != NULL ? fl_value_lookup_string(args, "exitLabel") : NULL;

  const gchar* icon_path =
      icon_value != NULL && fl_value_get_type(icon_value) == FL_VALUE_TYPE_STRING
          ? fl_value_get_string(icon_value)
          : NULL;
  const gchar* tooltip =
      tooltip_value != NULL && fl_value_get_type(tooltip_value) == FL_VALUE_TYPE_STRING
          ? fl_value_get_string(tooltip_value)
          : "CopyPaste";
  const gchar* show_hide =
      toggle_value != NULL && fl_value_get_type(toggle_value) == FL_VALUE_TYPE_STRING
          ? fl_value_get_string(toggle_value)
          : "Show/Hide";
  const gchar* exit_label =
      exit_value != NULL && fl_value_get_type(exit_value) == FL_VALUE_TYPE_STRING
          ? fl_value_get_string(exit_value)
          : "Exit";

  if (shell->tray_icon == NULL) {
    shell->tray_icon = gtk_status_icon_new();
    g_signal_connect(shell->tray_icon, "activate", G_CALLBACK(tray_activate_cb),
                     shell);
    g_signal_connect(shell->tray_icon, "popup-menu",
                     G_CALLBACK(tray_popup_menu_cb), shell);
  }

  g_free(shell->resolved_icon_path);
  shell->resolved_icon_path = resolve_asset_path(icon_path);
  if (shell->resolved_icon_path != NULL) {
    gtk_status_icon_set_from_file(shell->tray_icon, shell->resolved_icon_path);
  }

  gtk_status_icon_set_tooltip_text(shell->tray_icon, tooltip);
  gtk_status_icon_set_visible(shell->tray_icon, TRUE);
  rebuild_tray_menu(shell, show_hide, exit_label);
  return TRUE;
}

static gboolean destroy_tray(CopyPasteLinuxShell* shell) {
  destroy_tray_menu(shell);

  if (shell->tray_icon != NULL) {
    gtk_status_icon_set_visible(shell->tray_icon, FALSE);
    g_clear_object(&shell->tray_icon);
  }

  g_clear_pointer(&shell->resolved_icon_path, g_free);
  return TRUE;
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
  return NoSymbol;
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

static gboolean register_hotkey(CopyPasteLinuxShell* shell, FlValue* args) {
  if (!shell_is_x11() || shell->xdisplay == NULL) {
    return FALSE;
  }

  unregister_hotkey(shell);

  FlValue* key_value = args != NULL ? fl_value_lookup_string(args, "virtualKey") : NULL;
  gint64 virtual_key = key_value != NULL ? fl_value_get_int(key_value) : 0;
  KeySym keysym = virtual_key_to_keysym(virtual_key);
  if (keysym == NoSymbol) {
    return FALSE;
  }

  guint modifiers = compute_modifier_mask(args);
  if (modifiers == 0) {
    return FALSE;
  }

  KeyCode keycode = XKeysymToKeycode(shell->xdisplay, keysym);
  if (keycode == 0) {
    return FALSE;
  }

  for (guint i = 0; i < G_N_ELEMENTS(modifier_combinations); i++) {
    if (!trap_x11_grab(shell->xdisplay, shell->root_window, keycode,
                       modifiers | modifier_combinations[i])) {
      ungrab_hotkey_variants(shell->xdisplay, shell->root_window, keycode,
                             modifiers);
      return FALSE;
    }
  }
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
  return TRUE;
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
  if (!fl_method_call_respond_success(method_call, result, &error) && error != NULL) {
    g_warning("Failed to respond to linux shell method call: %s", error->message);
  }
}

static void shell_method_call_cb(FlMethodChannel* channel,
                                 FlMethodCall* method_call,
                                 gpointer user_data) {
  (void)channel;
  CopyPasteLinuxShell* shell = (CopyPasteLinuxShell*)user_data;
  const gchar* method = fl_method_call_get_name(method_call);
  FlValue* args = fl_method_call_get_args(method_call);

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
    respond_method_success(method_call, fl_value_new_bool(register_hotkey(shell, args)));
#else
    respond_method_success(method_call, fl_value_new_bool(FALSE));
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

  g_autoptr(FlMethodResponse) response =
      FL_METHOD_RESPONSE(fl_method_not_implemented_response_new());
  fl_method_call_respond(method_call, response, NULL);
}

CopyPasteLinuxShell* copypaste_linux_shell_new(FlBinaryMessenger* messenger,
                                               GtkWindow* window) {
  (void)window;
  CopyPasteLinuxShell* shell = g_new0(CopyPasteLinuxShell, 1);

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
