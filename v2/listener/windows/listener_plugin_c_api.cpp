#include "include/listener/listener_plugin_c_api.h"

#include <flutter/plugin_registrar_windows.h>

#include "listener_plugin.h"

void ListenerPluginCApiRegisterWithRegistrar(
    FlutterDesktopPluginRegistrarRef registrar) {
  listener::ListenerPlugin::RegisterWithRegistrar(
      flutter::PluginRegistrarManager::GetInstance()
          ->GetRegistrar<flutter::PluginRegistrarWindows>(registrar));
}
