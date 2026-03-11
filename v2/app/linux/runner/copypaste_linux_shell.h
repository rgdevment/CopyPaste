#ifndef FLUTTER_COPYPASTE_LINUX_SHELL_H_
#define FLUTTER_COPYPASTE_LINUX_SHELL_H_

#include <flutter_linux/flutter_linux.h>
#include <gtk/gtk.h>

G_BEGIN_DECLS

typedef struct _CopyPasteLinuxShell CopyPasteLinuxShell;

CopyPasteLinuxShell* copypaste_linux_shell_new(FlBinaryMessenger* messenger,
                                               GtkWindow* window);

void copypaste_linux_shell_dispose(CopyPasteLinuxShell* shell);

G_END_DECLS

#endif  // FLUTTER_COPYPASTE_LINUX_SHELL_H_