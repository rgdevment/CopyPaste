#include "flutter_window.h"

#include <flutter/standard_method_codec.h>

#include <cstdint>
#include <optional>
#include <string>

#include "flutter/generated_plugin_registrant.h"
#include "startup_task_channel.h"

namespace {

constexpr char kHotkeyChannelName[] = "copypaste/windows_hotkeys";
constexpr int kOpenHotkeyId = 0x4301;
constexpr int kPlainPasteHotkeyId = 0x4302;
constexpr UINT kModNoRepeat = 0x4000;

const flutter::EncodableValue* FindArgument(
    const flutter::EncodableMap& arguments, const char* name) {
  const auto it = arguments.find(flutter::EncodableValue(name));
  return it == arguments.end() ? nullptr : &it->second;
}

bool ReadBool(const flutter::EncodableMap& arguments, const char* name) {
  const auto* value = FindArgument(arguments, name);
  const auto* boolean = value == nullptr ? nullptr : std::get_if<bool>(value);
  return boolean != nullptr && *boolean;
}

bool ReadInt(const flutter::EncodableMap& arguments, const char* name,
             int* result) {
  const auto* value = FindArgument(arguments, name);
  if (value == nullptr) return false;
  if (const auto* int32 = std::get_if<int32_t>(value)) {
    *result = *int32;
    return true;
  }
  if (const auto* int64 = std::get_if<int64_t>(value)) {
    *result = static_cast<int>(*int64);
    return true;
  }
  return false;
}

std::string ReadString(const flutter::EncodableMap& arguments,
                       const char* name) {
  const auto* value = FindArgument(arguments, name);
  const auto* string =
      value == nullptr ? nullptr : std::get_if<std::string>(value);
  return string == nullptr ? std::string() : *string;
}

flutter::EncodableValue RegistrationResponse(bool success,
                                              DWORD win32_error = ERROR_SUCCESS) {
  flutter::EncodableMap response;
  response[flutter::EncodableValue("success")] =
      flutter::EncodableValue(success);
  if (!success) {
    response[flutter::EncodableValue("errorCode")] =
        flutter::EncodableValue("registerFailed");
    response[flutter::EncodableValue("win32Error")] =
        flutter::EncodableValue(static_cast<int64_t>(win32_error));
  }
  return flutter::EncodableValue(response);
}

}  // namespace

FlutterWindow::FlutterWindow(const flutter::DartProject& project)
    : project_(project) {}

FlutterWindow::~FlutterWindow() {}

bool FlutterWindow::OnCreate() {
  if (!Win32Window::OnCreate()) {
    return false;
  }

  RECT frame = GetClientArea();

  // The size here must match the window dimensions to avoid unnecessary surface
  // creation / destruction in the startup path.
  flutter_controller_ = std::make_unique<flutter::FlutterViewController>(
      frame.right - frame.left, frame.bottom - frame.top, project_);
  // Ensure that basic setup of the controller was successful.
  if (!flutter_controller_->engine() || !flutter_controller_->view()) {
    return false;
  }
  RegisterPlugins(flutter_controller_->engine());
  RegisterStartupTaskChannel(flutter_controller_.get());
  RegisterHotkeyChannel();
  SetChildContent(flutter_controller_->view()->GetNativeWindow());

  flutter_controller_->engine()->SetNextFrameCallback([&]() {
    // Window visibility is managed by window_manager plugin (Dart side).
    // Do NOT call Show() here — it causes a visible flash on startup.
  });

  // Flutter can complete the first frame before the "show window" callback is
  // registered. The following call ensures a frame is pending to ensure the
  // window is shown. It is a no-op if the first frame hasn't completed yet.
  flutter_controller_->ForceRedraw();

  return true;
}

void FlutterWindow::OnDestroy() {
  UnregisterAllHotkeys();
  if (hotkey_channel_) {
    // MethodChannel destruction does not unregister its messenger callback.
    // Remove the handler before releasing the lambda that captures this.
    hotkey_channel_->SetMethodCallHandler(nullptr);
  }
  hotkey_channel_.reset();
  if (flutter_controller_) {
    flutter_controller_ = nullptr;
  }

  Win32Window::OnDestroy();
}

LRESULT
FlutterWindow::MessageHandler(HWND hwnd, UINT const message,
                              WPARAM const wparam,
                              LPARAM const lparam) noexcept {
  if (message == WM_HOTKEY && hotkey_channel_) {
    const char* id = nullptr;
    if (static_cast<int>(wparam) == kOpenHotkeyId) {
      id = "open";
    } else if (static_cast<int>(wparam) == kPlainPasteHotkeyId) {
      id = "plainPaste";
    }
    if (id != nullptr) {
      hotkey_channel_->InvokeMethod(
          "hotkeyPressed",
          std::make_unique<flutter::EncodableValue>(std::string(id)));
      return 0;
    }
  }

  // Give Flutter, including plugins, an opportunity to handle window messages.
  if (flutter_controller_) {
    std::optional<LRESULT> result =
        flutter_controller_->HandleTopLevelWindowProc(hwnd, message, wparam,
                                                      lparam);
    if (result) {
      return *result;
    }
  }

  switch (message) {
    case WM_FONTCHANGE:
      if (flutter_controller_) {
        flutter_controller_->engine()->ReloadSystemFonts();
      }
      break;
  }

  return Win32Window::MessageHandler(hwnd, message, wparam, lparam);
}

void FlutterWindow::RegisterHotkeyChannel() {
  hotkey_channel_ =
      std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
          flutter_controller_->engine()->messenger(), kHotkeyChannelName,
          &flutter::StandardMethodCodec::GetInstance());

  hotkey_channel_->SetMethodCallHandler(
      [this](const flutter::MethodCall<flutter::EncodableValue>& call,
             std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>>
                 result) {
        if (call.method_name() == "unregisterAll") {
          UnregisterAllHotkeys();
          result->Success();
          return;
        }
        if (call.method_name() != "register") {
          result->NotImplemented();
          return;
        }

        const auto* arguments =
            std::get_if<flutter::EncodableMap>(call.arguments());
        int virtual_key = 0;
        if (arguments == nullptr ||
            !ReadInt(*arguments, "virtualKey", &virtual_key) ||
            virtual_key <= 0 || virtual_key > 0xFF) {
          result->Error("invalid_args", "A valid virtualKey is required");
          return;
        }

        const std::string id = ReadString(*arguments, "id");
        int native_id = 0;
        bool* is_registered = nullptr;
        if (id == "open") {
          native_id = kOpenHotkeyId;
          is_registered = &open_hotkey_registered_;
        } else if (id == "plainPaste") {
          native_id = kPlainPasteHotkeyId;
          is_registered = &plain_paste_hotkey_registered_;
        } else {
          result->Error("invalid_args", "Unknown hotkey id");
          return;
        }

        if (*is_registered) {
          UnregisterHotKey(GetHandle(), native_id);
          *is_registered = false;
        }

        UINT modifiers = kModNoRepeat;
        if (ReadBool(*arguments, "useCtrl")) modifiers |= MOD_CONTROL;
        if (ReadBool(*arguments, "useWin")) modifiers |= MOD_WIN;
        if (ReadBool(*arguments, "useAlt")) modifiers |= MOD_ALT;
        if (ReadBool(*arguments, "useShift")) modifiers |= MOD_SHIFT;

        if (!RegisterHotKey(GetHandle(), native_id, modifiers,
                            static_cast<UINT>(virtual_key))) {
          result->Success(RegistrationResponse(false, GetLastError()));
          return;
        }
        *is_registered = true;
        result->Success(RegistrationResponse(true));
      });
}

void FlutterWindow::UnregisterAllHotkeys() {
  if (open_hotkey_registered_) {
    UnregisterHotKey(GetHandle(), kOpenHotkeyId);
    open_hotkey_registered_ = false;
  }
  if (plain_paste_hotkey_registered_) {
    UnregisterHotKey(GetHandle(), kPlainPasteHotkeyId);
    plain_paste_hotkey_registered_ = false;
  }
}
