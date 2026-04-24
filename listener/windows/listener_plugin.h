#ifndef FLUTTER_PLUGIN_LISTENER_PLUGIN_H_
#define FLUTTER_PLUGIN_LISTENER_PLUGIN_H_

#include <flutter/event_channel.h>
#include <flutter/event_sink.h>
#include <flutter/method_channel.h>
#include <flutter/plugin_registrar_windows.h>
#include <flutter/standard_method_codec.h>

#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>

#include <objidl.h>
#include <gdiplus.h>

#include <memory>
#include <mutex>
#include <string>
#include <vector>

namespace listener {

class ListenerPlugin : public flutter::Plugin {
 public:
  static void RegisterWithRegistrar(flutter::PluginRegistrarWindows* registrar);

  explicit ListenerPlugin(flutter::PluginRegistrarWindows* registrar);
  ~ListenerPlugin() override;

  ListenerPlugin(const ListenerPlugin&) = delete;
  ListenerPlugin& operator=(const ListenerPlugin&) = delete;

  static std::string WideToUtf8(const std::wstring& wide);

  void StartListening(
      std::unique_ptr<flutter::EventSink<flutter::EncodableValue>> sink);
  void StopListening();

  void HandleMethodCall(
      const flutter::MethodCall<flutter::EncodableValue>& call,
      std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result);

 private:
  flutter::PluginRegistrarWindows* registrar_;
  int window_proc_id_ = -1;

  std::mutex sink_mutex_;
  std::unique_ptr<flutter::EventSink<flutter::EncodableValue>> sink_;

  std::string last_content_hash_;
  ULONGLONG last_change_tick_ = 0;
  static constexpr ULONGLONG kDebounceMs = 500;
  static constexpr UINT_PTR kClipboardTimerId = 1;
  static constexpr UINT kClipboardTimerDelayMs = 50;
  static constexpr ULONGLONG kSelfWriteIgnoreMs = 700;
  static constexpr int kOpenClipboardRetries = 3;
  static constexpr DWORD kOpenClipboardBackoffMs[] = {5, 10, 20};

  ULONGLONG last_write_tick_ = 0;

  UINT cf_rtf_ = 0;
  UINT cf_html_ = 0;
  UINT cf_exclude_history_ = 0;
  UINT cf_can_include_ = 0;
  ULONG_PTR gdip_token_ = 0;

  std::optional<LRESULT> HandleWindowMessage(HWND hwnd, UINT message,
                                              WPARAM wparam, LPARAM lparam);
  void OnClipboardChanged();
  bool ShouldExclude() const;
  bool IsDuplicate(const std::string& hash);
  std::string ComputeClipboardHash() const;
  std::string GetClipboardSource() const;

  static std::wstring ExtractText();
  static std::vector<uint8_t> ExtractBytes(UINT format);
  static std::vector<std::wstring> ExtractFilePaths();
  static bool IsUrl(const std::wstring& text);
  static int DetectFileType(const std::wstring& path);
  static std::wstring Utf8ToWide(const std::string& utf8);
  static std::string ComputeSimpleHash(const std::string& data);

  bool SetTextToClipboard(const std::string& text,
                          const std::vector<uint8_t>& rtf,
                          const std::vector<uint8_t>& html);
  bool SetImageToClipboard(const std::string& imagePath);
  bool SetFilesToClipboard(const std::vector<std::string>& paths);
};

}  // namespace listener

#endif  // FLUTTER_PLUGIN_LISTENER_PLUGIN_H_

