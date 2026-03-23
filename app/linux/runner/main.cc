#include "my_application.h"

int main(int argc, char** argv) {
  g_setenv("GDK_BACKEND", "x11", FALSE);
  g_autoptr(MyApplication) app = my_application_new();
  return g_application_run(G_APPLICATION(app), argc, argv);
}
