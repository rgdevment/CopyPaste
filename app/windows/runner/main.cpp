#include <flutter/dart_project.h>
#include <flutter/flutter_view_controller.h>
#include <windows.h>

#include "flutter_window.h"
#include "utils.h"

// Signals the running instance so it can show its window.
// Best-effort: failures are silently ignored.
static void SignalRunningInstance() {
  ::AllowSetForegroundWindow(ASFW_ANY);

  HANDLE hPipe = ::CreateFileW(
      L"\\\\.\\pipe\\CopyPasteSingleInstance",
      GENERIC_WRITE, 0, nullptr, OPEN_EXISTING, 0, nullptr);
  if (hPipe != INVALID_HANDLE_VALUE) {
    DWORD written = 0;
    ::WriteFile(hPipe, "wakeup", 6, &written, nullptr);
    ::CloseHandle(hPipe);
  } else {
    wchar_t tempPath[MAX_PATH];
    if (::GetTempPathW(MAX_PATH, tempPath) > 0) {
      wchar_t wakeupPath[MAX_PATH];
      swprintf_s(wakeupPath, MAX_PATH, L"%scopypaste.wakeup", tempPath);
      HANDLE hFile = ::CreateFileW(wakeupPath, GENERIC_WRITE, 0, nullptr,
          CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);
      if (hFile != INVALID_HANDLE_VALUE) {
        DWORD written = 0;
        ::WriteFile(hFile, "wakeup", 6, &written, nullptr);
        ::CloseHandle(hFile);
      }
    }
  }
}

int APIENTRY wWinMain(_In_ HINSTANCE instance, _In_opt_ HINSTANCE prev,
                      _In_ wchar_t *command_line, _In_ int show_command) {
  // Native single-instance guard: detect an existing instance BEFORE creating
  // the Flutter window so the user never sees a second window at all.
  // Uses OpenMutexW (check-only); the authoritative mutex is managed by Dart.
  HANDLE hExisting = ::OpenMutexW(SYNCHRONIZE, FALSE,
      L"Local\\CopyPaste_SingleInstance_Mutex");
  if (hExisting != nullptr) {
    ::CloseHandle(hExisting);
    SignalRunningInstance();
    return 0;
  }

  // Attach to console when present (e.g., 'flutter run') or create a
  // new console when running with a debugger.
  if (!::AttachConsole(ATTACH_PARENT_PROCESS) && ::IsDebuggerPresent()) {
    CreateAndAttachConsole();
  }

  // Initialize COM, so that it is available for use in the library and/or
  // plugins.
  ::CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);

  flutter::DartProject project(L"data");

  std::vector<std::string> command_line_arguments =
      GetCommandLineArguments();

  project.set_dart_entrypoint_arguments(std::move(command_line_arguments));

  FlutterWindow window(project);
  Win32Window::Point origin(10, 10);
  Win32Window::Size size(1280, 720);
  if (!window.Create(L"CopyPaste", origin, size)) {
    return EXIT_FAILURE;
  }
  window.SetQuitOnClose(true);

  ::MSG msg;
  while (::GetMessage(&msg, nullptr, 0, 0)) {
    ::TranslateMessage(&msg);
    ::DispatchMessage(&msg);
  }

  ::CoUninitialize();
  return EXIT_SUCCESS;
}
