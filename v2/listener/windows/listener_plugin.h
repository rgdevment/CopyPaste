#ifndef FLUTTER_PLUGIN_LISTENER_PLUGIN_H_
#define FLUTTER_PLUGIN_LISTENER_PLUGIN_H_

#include <flutter/method_channel.h>
#include <flutter/plugin_registrar_windows.h>

#include <memory>

namespace listener {

class ListenerPlugin : public flutter::Plugin {
 public:
  static void RegisterWithRegistrar(flutter::PluginRegistrarWindows *registrar);

  ListenerPlugin();

  virtual ~ListenerPlugin();

  // Disallow copy and assign.
  ListenerPlugin(const ListenerPlugin&) = delete;
  ListenerPlugin& operator=(const ListenerPlugin&) = delete;

  // Called when a method is called on this plugin's channel from Dart.
  void HandleMethodCall(
      const flutter::MethodCall<flutter::EncodableValue> &method_call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result);
};

}  // namespace listener

#endif  // FLUTTER_PLUGIN_LISTENER_PLUGIN_H_
