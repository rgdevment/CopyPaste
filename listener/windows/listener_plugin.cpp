#include "listener_plugin.h"

#include <flutter/event_channel.h>
#include <flutter/event_sink.h>
#include <flutter/method_channel.h>
#include <flutter/plugin_registrar_windows.h>
#include <flutter/standard_method_codec.h>
#include <flutter/encodable_value.h>

#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <shellapi.h>
#include <shlobj.h>
#include <propsys.h>
#include <propkey.h>
#include <propvarutil.h>
#include <objidl.h>
#include <gdiplus.h>

#pragma comment(lib, "gdiplus.lib")

#include <algorithm>
#include <cstring>
#include <filesystem>
#include <map>
#include <optional>
#include <unordered_map>
#include <sstream>
#include <string>
#include <vector>

#pragma comment(lib, "shell32.lib")
#pragma comment(lib, "propsys.lib")

namespace listener {

namespace {

std::vector<uint8_t> ConvertDibToBmp(const std::vector<uint8_t>& dib) {
  if (dib.size() < sizeof(BITMAPINFOHEADER)) return {};

  const auto* bih = reinterpret_cast<const BITMAPINFOHEADER*>(dib.data());
  DWORD colorTableSize = 0;
  if (bih->biBitCount <= 8) {
    DWORD colors = bih->biClrUsed ? bih->biClrUsed : (1u << bih->biBitCount);
    colorTableSize = colors * sizeof(RGBQUAD);
  } else if (bih->biCompression == BI_BITFIELDS &&
             bih->biSize == sizeof(BITMAPINFOHEADER)) {
    // BI_BITFIELDS masks only follow the header for the classic
    // BITMAPINFOHEADER (40 bytes). For BITMAPV4HEADER (108) and
    // BITMAPV5HEADER (124) — produced by the Windows Snipping Tool — the
    // masks are embedded inside the header itself, so no extra offset.
    colorTableSize = 3 * sizeof(DWORD);
  }

  BITMAPFILEHEADER bfh = {};
  bfh.bfType = 0x4D42;
  bfh.bfSize = static_cast<DWORD>(sizeof(BITMAPFILEHEADER) + dib.size());
  bfh.bfOffBits = sizeof(BITMAPFILEHEADER) + bih->biSize + colorTableSize;

  std::vector<uint8_t> bmp(sizeof(BITMAPFILEHEADER) + dib.size());
  std::memcpy(bmp.data(), &bfh, sizeof(BITMAPFILEHEADER));
  std::memcpy(bmp.data() + sizeof(BITMAPFILEHEADER), dib.data(), dib.size());
  return bmp;
}

flutter::EncodableMap GetMediaInfo(const std::wstring& filePath) {
  flutter::EncodableMap info;

  IPropertyStore* pStore = nullptr;
  HRESULT hr = SHGetPropertyStoreFromParsingName(
      filePath.c_str(), nullptr, GPS_DEFAULT, IID_PPV_ARGS(&pStore));
  if (FAILED(hr) || !pStore) return info;

  // Duration (100-nanosecond units → seconds as int)
  PROPVARIANT pv;
  PropVariantInit(&pv);
  if (SUCCEEDED(pStore->GetValue(PKEY_Media_Duration, &pv)) &&
      pv.vt == VT_UI8) {
    auto seconds =
        static_cast<int64_t>(pv.uhVal.QuadPart / 10000000ULL);
    info[flutter::EncodableValue("duration")] =
        flutter::EncodableValue(seconds);
  }
  PropVariantClear(&pv);

  // Video dimensions
  PropVariantInit(&pv);
  if (SUCCEEDED(pStore->GetValue(PKEY_Video_FrameWidth, &pv)) &&
      pv.vt == VT_UI4) {
    info[flutter::EncodableValue("video_width")] =
        flutter::EncodableValue(static_cast<int>(pv.ulVal));
  }
  PropVariantClear(&pv);

  PropVariantInit(&pv);
  if (SUCCEEDED(pStore->GetValue(PKEY_Video_FrameHeight, &pv)) &&
      pv.vt == VT_UI4) {
    info[flutter::EncodableValue("video_height")] =
        flutter::EncodableValue(static_cast<int>(pv.ulVal));
  }
  PropVariantClear(&pv);

  // Artist (album artist — single-valued, matches v1's FirstAlbumArtist)
  PropVariantInit(&pv);
  if (SUCCEEDED(pStore->GetValue(PKEY_Music_AlbumArtist, &pv))) {
    PWSTR str = nullptr;
    if (SUCCEEDED(PropVariantToStringAlloc(pv, &str)) && str) {
      if (wcslen(str) > 0) {
        info[flutter::EncodableValue("artist")] =
            flutter::EncodableValue(ListenerPlugin::WideToUtf8(std::wstring(str)));
      }
      CoTaskMemFree(str);
    }
  }
  PropVariantClear(&pv);

  // Title
  PropVariantInit(&pv);
  if (SUCCEEDED(pStore->GetValue(PKEY_Title, &pv))) {
    PWSTR str = nullptr;
    if (SUCCEEDED(PropVariantToStringAlloc(pv, &str)) && str) {
      if (wcslen(str) > 0) {
        info[flutter::EncodableValue("title")] =
            flutter::EncodableValue(ListenerPlugin::WideToUtf8(std::wstring(str)));
      }
      CoTaskMemFree(str);
    }
  }
  PropVariantClear(&pv);

  // Album
  PropVariantInit(&pv);
  if (SUCCEEDED(pStore->GetValue(PKEY_Music_AlbumTitle, &pv))) {
    PWSTR str = nullptr;
    if (SUCCEEDED(PropVariantToStringAlloc(pv, &str)) && str) {
      if (wcslen(str) > 0) {
        info[flutter::EncodableValue("album")] =
            flutter::EncodableValue(ListenerPlugin::WideToUtf8(std::wstring(str)));
      }
      CoTaskMemFree(str);
    }
  }
  PropVariantClear(&pv);

  pStore->Release();
  return info;
}

class ClipboardStreamHandler
    : public flutter::StreamHandler<flutter::EncodableValue> {
 public:
  explicit ClipboardStreamHandler(ListenerPlugin* plugin) : plugin_(plugin) {}

 protected:
  std::unique_ptr<flutter::StreamHandlerError<flutter::EncodableValue>>
  OnListenInternal(
      const flutter::EncodableValue* arguments,
      std::unique_ptr<flutter::EventSink<flutter::EncodableValue>>&&
          events) override {
    plugin_->StartListening(std::move(events));
    return nullptr;
  }

  std::unique_ptr<flutter::StreamHandlerError<flutter::EncodableValue>>
  OnCancelInternal(const flutter::EncodableValue* arguments) override {
    plugin_->StopListening();
    return nullptr;
  }

 private:
  ListenerPlugin* plugin_;
};

}  // namespace

void ListenerPlugin::RegisterWithRegistrar(
    flutter::PluginRegistrarWindows* registrar) {
  auto plugin = std::make_unique<ListenerPlugin>(registrar);

  auto channel =
      std::make_unique<flutter::EventChannel<flutter::EncodableValue>>(
          registrar->messenger(), "copypaste/clipboard",
          &flutter::StandardMethodCodec::GetInstance());

  auto handler = std::make_unique<ClipboardStreamHandler>(plugin.get());
  channel->SetStreamHandler(std::move(handler));

  auto method_channel =
      std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
          registrar->messenger(), "copypaste/clipboard_writer",
          &flutter::StandardMethodCodec::GetInstance());
  auto* plugin_ptr = plugin.get();
  method_channel->SetMethodCallHandler(
      [plugin_ptr](
          const flutter::MethodCall<flutter::EncodableValue>& call,
          std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>>
              result) {
        plugin_ptr->HandleMethodCall(call, std::move(result));
      });

  registrar->AddPlugin(std::move(plugin));
}

ListenerPlugin::ListenerPlugin(flutter::PluginRegistrarWindows* registrar)
    : registrar_(registrar) {
  cf_rtf_ = RegisterClipboardFormat(L"Rich Text Format");
  cf_html_ = RegisterClipboardFormat(L"HTML Format");
  cf_exclude_history_ = RegisterClipboardFormat(
      L"ExcludeClipboardContentFromMonitorProcessing");
  cf_can_include_ =
      RegisterClipboardFormat(L"CanIncludeInClipboardHistory");

  Gdiplus::GdiplusStartupInput gdipInput;
  Gdiplus::GdiplusStartup(&gdip_token_, &gdipInput, nullptr);
}

ListenerPlugin::~ListenerPlugin() {
  StopListening();
  if (gdip_token_) Gdiplus::GdiplusShutdown(gdip_token_);
}

void ListenerPlugin::StartListening(
    std::unique_ptr<flutter::EventSink<flutter::EncodableValue>> sink) {
  std::lock_guard<std::mutex> lock(sink_mutex_);
  sink_ = std::move(sink);

  HWND hwnd = registrar_->GetView()
                   ? registrar_->GetView()->GetNativeWindow()
                   : nullptr;
  // Use the top-level window for AddClipboardFormatListener so that
  // WM_CLIPBOARDUPDATE arrives at the same WndProc that dispatches
  // to RegisterTopLevelWindowProcDelegate callbacks.
  HWND topHwnd = hwnd ? GetAncestor(hwnd, GA_ROOT) : nullptr;
  if (topHwnd) {
    AddClipboardFormatListener(topHwnd);
  }

  window_proc_id_ = registrar_->RegisterTopLevelWindowProcDelegate(
      [this](HWND hwnd, UINT message, WPARAM wparam,
             LPARAM lparam) -> std::optional<LRESULT> {
        return HandleWindowMessage(hwnd, message, wparam, lparam);
      });
}

void ListenerPlugin::StopListening() {
  if (window_proc_id_ >= 0) {
    registrar_->UnregisterTopLevelWindowProcDelegate(window_proc_id_);
    window_proc_id_ = -1;
  }

  HWND hwnd = registrar_->GetView() ? registrar_->GetView()->GetNativeWindow()
                                     : nullptr;
  HWND topHwnd = hwnd ? GetAncestor(hwnd, GA_ROOT) : nullptr;
  if (topHwnd) {
    KillTimer(topHwnd, kClipboardTimerId);
    RemoveClipboardFormatListener(topHwnd);
  }

  std::lock_guard<std::mutex> lock(sink_mutex_);
  sink_ = nullptr;
}

std::optional<LRESULT> ListenerPlugin::HandleWindowMessage(
    HWND hwnd, UINT message, WPARAM wparam, LPARAM lparam) {
  if (message == WM_CLIPBOARDUPDATE) {
    KillTimer(hwnd, kClipboardTimerId);
    SetTimer(hwnd, kClipboardTimerId, kClipboardTimerDelayMs, nullptr);
  } else if (message == WM_TIMER && wparam == kClipboardTimerId) {
    KillTimer(hwnd, kClipboardTimerId);
    OnClipboardChanged();
  }
  return std::nullopt;
}

void ListenerPlugin::OnClipboardChanged() {
  HWND hwnd = registrar_->GetView() ? registrar_->GetView()->GetNativeWindow()
                                     : nullptr;
  if (!hwnd) return;
  if (last_write_tick_ > 0 &&
      (GetTickCount64() - last_write_tick_) < kSelfWriteIgnoreMs) {
    return;
  }

  // Retry OpenClipboard up to kOpenClipboardRetries times with backoff.
  // Another app may hold the clipboard lock briefly; retrying avoids silent
  // drops. On exhaustion, log and bail — the next WM_CLIPBOARDUPDATE retries.
  bool opened = false;
  for (int attempt = 0; attempt < kOpenClipboardRetries; ++attempt) {
    if (OpenClipboard(hwnd)) {
      opened = true;
      break;
    }
    Sleep(kOpenClipboardBackoffMs[attempt]);
  }
  if (!opened) {
    OutputDebugStringA("[ClipboardListener] OpenClipboard failed after retries\n");
    return;
  }

  flutter::EncodableMap event;

  try {
    if (ShouldExclude()) {
      CloseClipboard();
      return;
    }

    std::string hash = ComputeClipboardHash();
    if (!hash.empty() && IsDuplicate(hash)) {
      CloseClipboard();
      return;
    }

    std::string source = GetClipboardSource();

    if (IsClipboardFormatAvailable(CF_HDROP)) {
      auto files = ExtractFilePaths();
      if (!files.empty()) {
        flutter::EncodableList file_list;
        file_list.reserve(files.size());
        int event_type = 2;  // file

        if (files.size() == 1) {
          event_type = DetectFileType(files[0]);
        }

        for (const auto& f : files) {
          file_list.push_back(flutter::EncodableValue(WideToUtf8(f)));
        }

        event = {
            {flutter::EncodableValue("type"),
             flutter::EncodableValue(event_type)},
            {flutter::EncodableValue("files"),
             flutter::EncodableValue(file_list)},
            {flutter::EncodableValue("source"),
             flutter::EncodableValue(source)},
            {flutter::EncodableValue("contentHash"),
             flutter::EncodableValue(hash)},
        };
      }
    } else if (IsClipboardFormatAvailable(CF_UNICODETEXT)) {
      std::wstring text = ExtractText();
      if (!text.empty()) {
        int event_type = IsUrl(text) ? 4 : 0;  // link=4, text=0

        std::vector<uint8_t> rtf_bytes;
        std::vector<uint8_t> html_bytes;
        if (cf_rtf_ && IsClipboardFormatAvailable(cf_rtf_)) {
          rtf_bytes = ExtractBytes(cf_rtf_);
        }
        if (cf_html_ && IsClipboardFormatAvailable(cf_html_)) {
          html_bytes = ExtractBytes(cf_html_);
        }

        event = {
            {flutter::EncodableValue("type"),
             flutter::EncodableValue(event_type)},
            {flutter::EncodableValue("text"),
             flutter::EncodableValue(WideToUtf8(text))},
            {flutter::EncodableValue("source"),
             flutter::EncodableValue(source)},
            {flutter::EncodableValue("contentHash"),
             flutter::EncodableValue(hash)},
        };
        if (!rtf_bytes.empty()) {
          event[flutter::EncodableValue("rtf")] =
              flutter::EncodableValue(rtf_bytes);
        }
        if (!html_bytes.empty()) {
          event[flutter::EncodableValue("html")] =
              flutter::EncodableValue(html_bytes);
        }
      }
    } else if (IsClipboardFormatAvailable(CF_DIB)) {
      auto dib = ExtractBytes(CF_DIB);
      if (!dib.empty()) {
        auto bytes = ConvertDibToBmp(dib);
        if (!bytes.empty()) {
        event = {
            {flutter::EncodableValue("type"),
             flutter::EncodableValue(1)},  // image=1
            {flutter::EncodableValue("bytes"),
             flutter::EncodableValue(bytes)},
            {flutter::EncodableValue("source"),
             flutter::EncodableValue(source)},
            {flutter::EncodableValue("contentHash"),
             flutter::EncodableValue(hash)},
        };
        }
      }
    }
  } catch (const std::exception& e) {
    std::cerr << "[CopyPaste Listener] Clipboard read error: " << e.what()
              << std::endl;
  } catch (...) {
    std::cerr << "[CopyPaste Listener] Unknown clipboard read error"
              << std::endl;
  }

  CloseClipboard();

  if (!event.empty()) {
    std::lock_guard<std::mutex> lock(sink_mutex_);
    if (sink_) {
      sink_->Success(flutter::EncodableValue(event));
    }
  }
}

bool ListenerPlugin::ShouldExclude() const {
  if (cf_exclude_history_ && IsClipboardFormatAvailable(cf_exclude_history_)) {
    return true;
  }
  if (cf_can_include_ && IsClipboardFormatAvailable(cf_can_include_)) {
    HANDLE hData = GetClipboardData(cf_can_include_);
    if (hData && GlobalSize(hData) >= 4) {
      const void* ptr = GlobalLock(hData);
      if (ptr) {
        const auto* bytes = static_cast<const uint8_t*>(ptr);
        int val = static_cast<int>(bytes[0]) | (static_cast<int>(bytes[1]) << 8) |
                  (static_cast<int>(bytes[2]) << 16) |
                  (static_cast<int>(bytes[3]) << 24);
        GlobalUnlock(hData);
        if (val == 0) return true;
      }
    }
  }
  return false;
}

bool ListenerPlugin::IsDuplicate(const std::string& hash) {
  ULONGLONG now = GetTickCount64();
  if (hash == last_content_hash_ && (now - last_change_tick_) < kDebounceMs) {
    return true;
  }
  last_content_hash_ = hash;
  last_change_tick_ = now;
  return false;
}

std::string ListenerPlugin::ComputeClipboardHash() const {
  std::string signature;

  if (IsClipboardFormatAvailable(CF_UNICODETEXT)) {
    std::wstring text = ExtractText();
    if (!text.empty()) {
      std::wstring sample = text.size() > 100 ? text.substr(0, 100) : text;
      signature += "T:" + WideToUtf8(sample);
    }
  } else if (IsClipboardFormatAvailable(CF_HDROP)) {
    auto files = ExtractFilePaths();
    for (const auto& f : files) {
      signature += "F:" + WideToUtf8(f) + "|";
    }
  } else if (IsClipboardFormatAvailable(CF_DIB)) {
    HANDLE hData = GetClipboardData(CF_DIB);
    if (hData) {
      SIZE_T sz = GlobalSize(hData);
      void* ptr = GlobalLock(hData);
      if (ptr) {
        size_t sample = (std::min)(sz, static_cast<SIZE_T>(256));
        std::ostringstream oss;
        oss << "I:" << sz << ":";
        const uint8_t* bytes = static_cast<const uint8_t*>(ptr);
        for (size_t i = 0; i < sample; ++i) {
          oss << std::hex << static_cast<int>(bytes[i]);
        }
        signature += oss.str();
        GlobalUnlock(hData);
      }
    }
  }

  if (signature.empty()) return {};
  return ComputeSimpleHash(signature);
}

std::string ListenerPlugin::GetClipboardSource() const {
  HWND owner = GetClipboardOwner();
  if (!owner) return {};

  DWORD pid = 0;
  GetWindowThreadProcessId(owner, &pid);
  if (!pid) return {};

  HANDLE proc =
      OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
  if (!proc) return {};

  wchar_t name[MAX_PATH] = {};
  DWORD size = MAX_PATH;
  QueryFullProcessImageNameW(proc, 0, name, &size);
  CloseHandle(proc);

  std::filesystem::path p(name);
  return WideToUtf8(p.stem().wstring());
}

std::wstring ListenerPlugin::ExtractText() {
  HANDLE hData = GetClipboardData(CF_UNICODETEXT);
  if (!hData) return {};

  void* ptr = GlobalLock(hData);
  if (!ptr) return {};

  SIZE_T sz = GlobalSize(hData);
  size_t maxChars = sz / sizeof(wchar_t);
  if (maxChars == 0) {
    GlobalUnlock(hData);
    return {};
  }

  const wchar_t* wptr = static_cast<const wchar_t*>(ptr);
  size_t len = wcsnlen(wptr, maxChars);
  std::wstring text(wptr, len);
  GlobalUnlock(hData);
  return text;
}

std::vector<uint8_t> ListenerPlugin::ExtractBytes(UINT format) {
  HANDLE hData = GetClipboardData(format);
  if (!hData) return {};

  void* ptr = GlobalLock(hData);
  if (!ptr) return {};

  SIZE_T sz = GlobalSize(hData);
  std::vector<uint8_t> result(sz);
  memcpy(result.data(), ptr, sz);
  GlobalUnlock(hData);
  return result;
}

std::vector<std::wstring> ListenerPlugin::ExtractFilePaths() {
  HANDLE hData = GetClipboardData(CF_HDROP);
  if (!hData) return {};

  UINT count = DragQueryFileW(static_cast<HDROP>(hData), 0xFFFFFFFF,
                               nullptr, 0);
  if (count == 0 || count > 10000) return {};

  std::vector<std::wstring> files;
  files.reserve(count);

  for (UINT i = 0; i < count; ++i) {
    UINT len = DragQueryFileW(static_cast<HDROP>(hData), i, nullptr, 0);
    if (len == 0 || len > 32767) continue;

    std::vector<wchar_t> buf(len + 1, L'\0');
    if (DragQueryFileW(static_cast<HDROP>(hData), i, buf.data(),
                       static_cast<UINT>(buf.size())) > 0) {
      std::wstring path(buf.data());
      if (!path.empty()) files.push_back(std::move(path));
    }
  }

  return files;
}

bool ListenerPlugin::IsUrl(const std::wstring& text) {
  if (text.size() < 5) return false;

  static const std::wstring kPrefixes[] = {
      L"https://", L"http://", L"ftp://", L"file:///", L"mailto:",
  };

  static constexpr size_t kMaxPrefix = 9;  // longest prefix is "file:///"
  const size_t checkLen = (std::min)(text.size(), kMaxPrefix);
  std::wstring head(text.data(), checkLen);
  std::transform(head.begin(), head.end(), head.begin(), ::towlower);

  bool matched = false;
  for (const auto& prefix : kPrefixes) {
    if (head.size() >= prefix.size() &&
        head.compare(0, prefix.size(), prefix) == 0) {
      matched = true;
      break;
    }
  }
  if (!matched) return false;

  return text.find(L' ') == std::wstring::npos &&
         text.find(L'\n') == std::wstring::npos;
}

int ListenerPlugin::DetectFileType(const std::wstring& path) {
  DWORD attrs = GetFileAttributesW(path.c_str());
  if (attrs != INVALID_FILE_ATTRIBUTES &&
      (attrs & FILE_ATTRIBUTE_DIRECTORY)) {
    return 3;  // folder
  }

  std::filesystem::path p(path);
  std::wstring ext = p.extension().wstring();
  std::transform(ext.begin(), ext.end(), ext.begin(), ::towupper);

  static const std::unordered_map<std::wstring, int> kExtMap = {
      {L".MP3", 5},  {L".WAV", 5},  {L".FLAC", 5}, {L".AAC", 5},
      {L".OGG", 5},  {L".WMA", 5},  {L".M4A", 5},
      {L".MP4", 6},  {L".AVI", 6},  {L".MKV", 6},  {L".MOV", 6},
      {L".WMV", 6},  {L".FLV", 6},  {L".WEBM", 6},
      {L".PNG", 1},  {L".JPG", 1},  {L".JPEG", 1}, {L".GIF", 1},
      {L".BMP", 1},  {L".WEBP", 1}, {L".SVG", 1},  {L".ICO", 1},
  };

  auto it = kExtMap.find(ext);
  return it != kExtMap.end() ? it->second : 2;  // default: file
}

std::string ListenerPlugin::WideToUtf8(const std::wstring& wide) {
  if (wide.empty()) return {};
  int sz = WideCharToMultiByte(CP_UTF8, 0, wide.data(),
                                static_cast<int>(wide.size()),
                                nullptr, 0, nullptr, nullptr);
  if (sz <= 0) return {};
  std::string result(sz, '\0');
  WideCharToMultiByte(CP_UTF8, 0, wide.data(),
                      static_cast<int>(wide.size()),
                      result.data(), sz, nullptr, nullptr);
  return result;
}

std::string ListenerPlugin::ComputeSimpleHash(const std::string& data) {
  // FNV-1a 64-bit hash
  uint64_t hash = 14695981039346656037ULL;
  for (unsigned char c : data) {
    hash ^= c;
    hash *= 1099511628211ULL;
  }
  std::ostringstream oss;
  oss << std::hex << hash;
  return oss.str();
}

std::wstring ListenerPlugin::Utf8ToWide(const std::string& utf8) {
  if (utf8.empty()) return {};
  int sz = MultiByteToWideChar(CP_UTF8, 0, utf8.data(),
                                static_cast<int>(utf8.size()),
                                nullptr, 0);
  if (sz <= 0) return {};
  std::wstring result(sz, L'\0');
  MultiByteToWideChar(CP_UTF8, 0, utf8.data(),
                      static_cast<int>(utf8.size()),
                      result.data(), sz);
  return result;
}

void ListenerPlugin::HandleMethodCall(
    const flutter::MethodCall<flutter::EncodableValue>& call,
    std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
  if (call.method_name() == "getMediaInfo") {
    const auto* args =
        std::get_if<flutter::EncodableMap>(call.arguments());
    if (!args) {
      result->Success(flutter::EncodableValue());
      return;
    }
    auto path_it = args->find(flutter::EncodableValue("path"));
    if (path_it == args->end()) {
      result->Success(flutter::EncodableValue());
      return;
    }
    std::string pathUtf8 = std::get<std::string>(path_it->second);
    auto info = GetMediaInfo(Utf8ToWide(pathUtf8));
    if (info.empty()) {
      result->Success(flutter::EncodableValue());
    } else {
      result->Success(flutter::EncodableValue(info));
    }
    return;
  }

  if (call.method_name() != "setClipboardContent") {
    result->NotImplemented();
    return;
  }

  const auto* args =
      std::get_if<flutter::EncodableMap>(call.arguments());
  if (!args) {
    result->Error("invalid_args", "Expected map arguments");
    return;
  }

  auto type_it = args->find(flutter::EncodableValue("type"));
  if (type_it == args->end()) {
    result->Error("missing_type", "Missing 'type' argument");
    return;
  }
  int type = std::get<int>(type_it->second);

  bool success = false;

  if (type == 0 || type == 4) {  // text or link
    auto content_it = args->find(flutter::EncodableValue("content"));
    std::string content =
        content_it != args->end()
            ? std::get<std::string>(content_it->second)
            : "";

    std::vector<uint8_t> rtf;
    auto rtf_it = args->find(flutter::EncodableValue("rtf"));
    if (rtf_it != args->end()) {
      rtf = std::get<std::vector<uint8_t>>(rtf_it->second);
    }

    std::vector<uint8_t> html;
    auto html_it = args->find(flutter::EncodableValue("html"));
    if (html_it != args->end()) {
      html = std::get<std::vector<uint8_t>>(html_it->second);
    }

    bool plain = (type == 4);
    auto plain_it = args->find(flutter::EncodableValue("plainText"));
    if (plain_it != args->end()) {
      plain = std::get<bool>(plain_it->second);
    }

    if (plain) {
      success = SetTextToClipboard(content, {}, {});
    } else {
      success = SetTextToClipboard(content, rtf, html);
    }
  } else if (type == 1) {  // image
    auto content_it = args->find(flutter::EncodableValue("content"));
    std::string imagePath =
        content_it != args->end()
            ? std::get<std::string>(content_it->second)
            : "";
    success = SetImageToClipboard(imagePath);
  } else if (type >= 2 && type <= 6) {  // file, folder, audio, video
    auto content_it = args->find(flutter::EncodableValue("content"));
    std::string content =
        content_it != args->end()
            ? std::get<std::string>(content_it->second)
            : "";
    std::vector<std::string> paths;
    std::istringstream iss(content);
    std::string line;
    while (std::getline(iss, line)) {
      if (!line.empty()) paths.push_back(line);
    }
    success = SetFilesToClipboard(paths);
  }

  result->Success(flutter::EncodableValue(success));
}

bool ListenerPlugin::SetTextToClipboard(
    const std::string& text,
    const std::vector<uint8_t>& rtf,
    const std::vector<uint8_t>& html) {
  if (text.empty()) return false;

  HWND hwnd = registrar_->GetView()
                  ? registrar_->GetView()->GetNativeWindow()
                  : nullptr;
  if (!OpenClipboard(hwnd)) return false;

  EmptyClipboard();
  bool ok = false;

  std::wstring wide = Utf8ToWide(text);
  if (wide.empty()) {
    CloseClipboard();
    return false;
  }
  size_t sz = (wide.size() + 1) * sizeof(wchar_t);
  HGLOBAL hMem = GlobalAlloc(GMEM_MOVEABLE, sz);
  if (hMem) {
    void* ptr = GlobalLock(hMem);
    if (ptr) {
      memcpy(ptr, wide.c_str(), sz);
      GlobalUnlock(hMem);
      if (SetClipboardData(CF_UNICODETEXT, hMem)) {
        ok = true;
      } else {
        GlobalFree(hMem);
      }
    } else {
      GlobalFree(hMem);
    }
  }

  if (ok && !rtf.empty() && cf_rtf_) {
    HGLOBAL hRtf = GlobalAlloc(GMEM_MOVEABLE, rtf.size() + 1);
    if (hRtf) {
      void* ptr = GlobalLock(hRtf);
      if (ptr) {
        memcpy(ptr, rtf.data(), rtf.size());
        static_cast<char*>(ptr)[rtf.size()] = '\0';
        GlobalUnlock(hRtf);
        if (!SetClipboardData(cf_rtf_, hRtf)) {
          GlobalFree(hRtf);
        }
      } else {
        GlobalFree(hRtf);
      }
    }
  }

  if (ok && !html.empty() && cf_html_) {
    HGLOBAL hHtml = GlobalAlloc(GMEM_MOVEABLE, html.size() + 1);
    if (hHtml) {
      void* ptr = GlobalLock(hHtml);
      if (ptr) {
        memcpy(ptr, html.data(), html.size());
        static_cast<char*>(ptr)[html.size()] = '\0';
        GlobalUnlock(hHtml);
        if (!SetClipboardData(cf_html_, hHtml)) {
          GlobalFree(hHtml);
        }
      } else {
        GlobalFree(hHtml);
      }
    }
  }

  CloseClipboard();
  if (ok) last_write_tick_ = GetTickCount64();
  return ok;
}

bool ListenerPlugin::SetImageToClipboard(const std::string& imagePath) {
  if (imagePath.empty()) return false;

  std::wstring wpath = Utf8ToWide(imagePath);

  // Use GDI+ to load any image format (PNG, BMP, JPEG, etc.)
  Gdiplus::Bitmap bitmap(wpath.c_str());
  if (bitmap.GetLastStatus() != Gdiplus::Ok) return false;

  HBITMAP hBitmap = nullptr;
  Gdiplus::Color bg(0, 255, 255, 255);
  if (bitmap.GetHBITMAP(bg, &hBitmap) != Gdiplus::Ok || !hBitmap)
    return false;

  BITMAP bm = {};
  GetObject(hBitmap, sizeof(bm), &bm);

  BITMAPINFOHEADER bih = {};
  bih.biSize = sizeof(BITMAPINFOHEADER);
  bih.biWidth = bm.bmWidth;
  bih.biHeight = bm.bmHeight;
  bih.biPlanes = 1;
  bih.biBitCount = 32;
  bih.biCompression = BI_RGB;

  size_t rowBytes = static_cast<size_t>(bm.bmWidth) * 4;
  size_t imgSize = rowBytes * bm.bmHeight;
  bih.biSizeImage = static_cast<DWORD>(imgSize);

  size_t dibSize = sizeof(BITMAPINFOHEADER) + imgSize;
  HGLOBAL hMem = GlobalAlloc(GMEM_MOVEABLE, dibSize);
  if (!hMem) {
    DeleteObject(hBitmap);
    return false;
  }

  void* ptr = GlobalLock(hMem);
  if (!ptr) {
    GlobalFree(hMem);
    DeleteObject(hBitmap);
    return false;
  }

  memcpy(ptr, &bih, sizeof(BITMAPINFOHEADER));

  HDC hDC = GetDC(nullptr);
  auto* bi = reinterpret_cast<BITMAPINFO*>(ptr);
  int scanLines = GetDIBits(hDC, hBitmap, 0, bm.bmHeight,
            static_cast<uint8_t*>(ptr) + sizeof(BITMAPINFOHEADER),
            bi, DIB_RGB_COLORS);
  ReleaseDC(nullptr, hDC);
  GlobalUnlock(hMem);
  DeleteObject(hBitmap);

  if (scanLines == 0) {
    GlobalFree(hMem);
    return false;
  }

  HWND hwnd = registrar_->GetView()
                  ? registrar_->GetView()->GetNativeWindow()
                  : nullptr;
  if (!OpenClipboard(hwnd)) {
    GlobalFree(hMem);
    return false;
  }

  EmptyClipboard();
  bool ok = SetClipboardData(CF_DIB, hMem) != nullptr;
  if (!ok) GlobalFree(hMem);

  if (ok) {
    DWORD attr = GetFileAttributesW(wpath.c_str());
    if (attr != INVALID_FILE_ATTRIBUTES) {
      size_t pathBytes = (wpath.size() + 1) * sizeof(wchar_t);
      size_t dropSize = sizeof(DROPFILES) + pathBytes + sizeof(wchar_t);
      HGLOBAL hDrop = GlobalAlloc(GHND, dropSize);
      if (hDrop) {
        auto* df = static_cast<DROPFILES*>(GlobalLock(hDrop));
        if (df) {
          df->pFiles = sizeof(DROPFILES);
          df->fWide = TRUE;
          auto* dest = reinterpret_cast<wchar_t*>(
              reinterpret_cast<uint8_t*>(df) + sizeof(DROPFILES));
          memcpy(dest, wpath.c_str(), pathBytes);
          GlobalUnlock(hDrop);
          if (!SetClipboardData(CF_HDROP, hDrop)) {
            GlobalFree(hDrop);
          }
        } else {
          GlobalFree(hDrop);
        }
      }
    }
  }

  CloseClipboard();
  if (ok) last_write_tick_ = GetTickCount64();
  return ok;
}

bool ListenerPlugin::SetFilesToClipboard(
    const std::vector<std::string>& paths) {
  if (paths.empty()) return false;

  std::vector<std::wstring> wpaths;
  wpaths.reserve(paths.size());
  size_t totalChars = 0;
  for (const auto& p : paths) {
    auto wp = Utf8ToWide(p);
    if (wp.empty()) continue;
    DWORD attr = GetFileAttributesW(wp.c_str());
    if (attr == INVALID_FILE_ATTRIBUTES) continue;
    totalChars += wp.size() + 1;
    wpaths.push_back(std::move(wp));
  }

  if (wpaths.empty()) return false;

  totalChars += 1;  // double null terminator
  size_t sz = sizeof(DROPFILES) + totalChars * sizeof(wchar_t);
  HGLOBAL hMem = GlobalAlloc(GHND, sz);
  if (!hMem) return false;

  auto* df = static_cast<DROPFILES*>(GlobalLock(hMem));
  if (!df) {
    GlobalFree(hMem);
    return false;
  }

  df->pFiles = sizeof(DROPFILES);
  df->fWide = TRUE;

  auto* dest = reinterpret_cast<wchar_t*>(
      reinterpret_cast<uint8_t*>(df) + sizeof(DROPFILES));
  for (const auto& wp : wpaths) {
    memcpy(dest, wp.c_str(), (wp.size() + 1) * sizeof(wchar_t));
    dest += wp.size() + 1;
  }
  *dest = L'\0';

  GlobalUnlock(hMem);

  HWND hwnd = registrar_->GetView()
                  ? registrar_->GetView()->GetNativeWindow()
                  : nullptr;
  if (!OpenClipboard(hwnd)) {
    GlobalFree(hMem);
    return false;
  }

  EmptyClipboard();
  bool ok = SetClipboardData(CF_HDROP, hMem) != nullptr;
  if (!ok) GlobalFree(hMem);

  CloseClipboard();
  if (ok) last_write_tick_ = GetTickCount64();
  return ok;
}

}  // namespace listener

