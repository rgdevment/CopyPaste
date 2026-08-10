#include "flutter_window.h"

#include <flutter/standard_method_codec.h>

#include <cstdint>
#include <optional>
#include <string>
#include <vector>

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

int64_t ReadInt64(const flutter::EncodableMap& arguments, const char* name) {
  const auto* value = FindArgument(arguments, name);
  if (value == nullptr) return 0;
  if (const auto* int32 = std::get_if<int32_t>(value)) return *int32;
  if (const auto* int64 = std::get_if<int64_t>(value)) return *int64;
  return 0;
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

// Mandatory integrity level of a process, or 0 when it cannot be read.
DWORD ProcessIntegrityLevel(DWORD pid) {
  if (pid == 0) return 0;
  HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
  if (process == nullptr) return 0;

  DWORD level = 0;
  HANDLE token = nullptr;
  if (OpenProcessToken(process, TOKEN_QUERY, &token)) {
    DWORD size = 0;
    GetTokenInformation(token, TokenIntegrityLevel, nullptr, 0, &size);
    if (size > 0) {
      std::vector<uint8_t> buffer(size);
      if (GetTokenInformation(token, TokenIntegrityLevel, buffer.data(), size,
                              &size)) {
        auto* label = reinterpret_cast<TOKEN_MANDATORY_LABEL*>(buffer.data());
        const UCHAR* count = GetSidSubAuthorityCount(label->Label.Sid);
        if (count != nullptr && *count > 0) {
          level = *GetSidSubAuthority(label->Label.Sid, *count - 1);
        }
      }
    }
    CloseHandle(token);
  }
  CloseHandle(process);
  return level;
}

// An active window is not the same as a usable focus: Chromium hosts (VS Code
// webviews) and XAML-island hosts (Windows Terminal) restore the focus of their
// inner child HWND asynchronously after WM_ACTIVATE, so a Ctrl+V timed on
// activation alone lands nowhere while every call still reports success.
// Attaching to the destination's input queue lets SetFocus target that child
// directly; the detach must happen after SendInput, because detaching resets
// the keyboard focus Windows had just restored.
flutter::EncodableValue SendPasteInput(HWND target, HWND target_focus,
                                       DWORD target_thread) {
  flutter::EncodableMap response;
  if (target != nullptr && GetForegroundWindow() != target) {
    response[flutter::EncodableValue("success")] =
        flutter::EncodableValue(false);
    response[flutter::EncodableValue("errorCode")] =
        flutter::EncodableValue("targetNotForeground");
    return flutter::EncodableValue(response);
  }

  // UIPI drops injected input at a higher integrity level and reports nothing:
  // SendInput still returns the full count. Without this check an elevated
  // destination is a permanent, undiagnosable "paste does nothing".
  if (target != nullptr) {
    DWORD target_pid = 0;
    GetWindowThreadProcessId(target, &target_pid);
    const DWORD target_level = ProcessIntegrityLevel(target_pid);
    const DWORD self_level = ProcessIntegrityLevel(GetCurrentProcessId());
    if (target_level != 0 && self_level != 0 && target_level > self_level) {
      response[flutter::EncodableValue("success")] =
          flutter::EncodableValue(false);
      response[flutter::EncodableValue("errorCode")] =
          flutter::EncodableValue("targetElevated");
      return flutter::EncodableValue(response);
    }
  }

  const DWORD self_thread = GetCurrentThreadId();
  const bool attached = target_thread != 0 && target_thread != self_thread &&
                        AttachThreadInput(self_thread, target_thread, TRUE);

  bool focus_repaired = false;
  HWND focus_before = nullptr;
  if (attached) {
    GUITHREADINFO gui = {};
    gui.cbSize = sizeof(gui);
    if (GetGUIThreadInfo(target_thread, &gui)) {
      focus_before = gui.hwndFocus;
    }
    if (target_focus != nullptr && focus_before != target_focus &&
        IsWindow(target_focus)) {
      // SetFocus on a foreign HWND is a blocking cross-thread send, so probe
      // the destination first: a hung target would otherwise freeze our UI.
      DWORD_PTR probe = 0;
      if (SendMessageTimeoutW(target_focus, WM_NULL, 0, 0,
                              SMTO_ABORTIFHUNG | SMTO_BLOCK, 200,
                              &probe) != 0) {
        focus_repaired = SetFocus(target_focus) != nullptr;
      }
    }
  }

  // WM_HOTKEY arrives on key-down, so physical shortcut modifiers may still
  // be held. Release every contaminating modifier before Ctrl+V. SendInput
  // inserts this array atomically; later physical key-up events are harmless.
  INPUT inputs[9] = {};
  inputs[0].type = INPUT_KEYBOARD;
  inputs[0].ki.wVk = VK_MENU;
  inputs[0].ki.dwFlags = KEYEVENTF_KEYUP;
  inputs[1].type = INPUT_KEYBOARD;
  inputs[1].ki.wVk = VK_SHIFT;
  inputs[1].ki.dwFlags = KEYEVENTF_KEYUP;
  inputs[2].type = INPUT_KEYBOARD;
  inputs[2].ki.wVk = VK_LWIN;
  inputs[2].ki.dwFlags = KEYEVENTF_KEYUP;
  inputs[3].type = INPUT_KEYBOARD;
  inputs[3].ki.wVk = VK_RWIN;
  inputs[3].ki.dwFlags = KEYEVENTF_KEYUP;
  inputs[4].type = INPUT_KEYBOARD;
  inputs[4].ki.wVk = VK_CONTROL;
  inputs[4].ki.dwFlags = KEYEVENTF_KEYUP;
  inputs[5].type = INPUT_KEYBOARD;
  inputs[5].ki.wVk = VK_CONTROL;
  inputs[6].type = INPUT_KEYBOARD;
  inputs[6].ki.wVk = 'V';
  inputs[7].type = INPUT_KEYBOARD;
  inputs[7].ki.wVk = 'V';
  inputs[7].ki.dwFlags = KEYEVENTF_KEYUP;
  inputs[8].type = INPUT_KEYBOARD;
  inputs[8].ki.wVk = VK_CONTROL;
  inputs[8].ki.dwFlags = KEYEVENTF_KEYUP;

  constexpr UINT kInputCount = sizeof(inputs) / sizeof(inputs[0]);

  SetLastError(ERROR_SUCCESS);
  const UINT sent = SendInput(kInputCount, inputs, sizeof(INPUT));
  const DWORD send_error = GetLastError();

  if (attached) {
    AttachThreadInput(self_thread, target_thread, FALSE);
  }

  response[flutter::EncodableValue("success")] =
      flutter::EncodableValue(sent == kInputCount);
  response[flutter::EncodableValue("sentInputs")] =
      flutter::EncodableValue(static_cast<int32_t>(sent));
  response[flutter::EncodableValue("expectedInputs")] =
      flutter::EncodableValue(static_cast<int32_t>(kInputCount));
  response[flutter::EncodableValue("attached")] =
      flutter::EncodableValue(attached);
  response[flutter::EncodableValue("focusRepaired")] =
      flutter::EncodableValue(focus_repaired);
  response[flutter::EncodableValue("focusBefore")] = flutter::EncodableValue(
      static_cast<int64_t>(reinterpret_cast<intptr_t>(focus_before)));
  if (sent != kInputCount) {
    response[flutter::EncodableValue("errorCode")] =
        flutter::EncodableValue("sendInputFailed");
    response[flutter::EncodableValue("win32Error")] =
        flutter::EncodableValue(static_cast<int64_t>(send_error));
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
        if (call.method_name() == "sendPaste") {
          const auto* paste_args =
              std::get_if<flutter::EncodableMap>(call.arguments());
          HWND target = nullptr;
          HWND target_focus = nullptr;
          DWORD target_thread = 0;
          if (paste_args != nullptr) {
            target = reinterpret_cast<HWND>(
                static_cast<intptr_t>(ReadInt64(*paste_args, "targetHwnd")));
            target_focus = reinterpret_cast<HWND>(static_cast<intptr_t>(
                ReadInt64(*paste_args, "targetFocusHwnd")));
            target_thread = static_cast<DWORD>(
                ReadInt64(*paste_args, "targetThreadId"));
          }
          result->Success(SendPasteInput(target, target_focus, target_thread));
          return;
        }
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
