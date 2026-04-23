#include "startup_task_channel.h"

#include <flutter/method_channel.h>
#include <flutter/standard_method_codec.h>

#include <winrt/Windows.ApplicationModel.h>
#include <winrt/Windows.Foundation.h>

#include <memory>
#include <string>
#include <thread>

namespace {

constexpr const char kChannelName[] = "copypaste/startup_task";

std::string StateToString(winrt::Windows::ApplicationModel::StartupTaskState state) {
  using winrt::Windows::ApplicationModel::StartupTaskState;
  switch (state) {
    case StartupTaskState::Disabled:
      return "disabled";
    case StartupTaskState::DisabledByUser:
      return "disabledByUser";
    case StartupTaskState::DisabledByPolicy:
      return "disabledByPolicy";
    case StartupTaskState::Enabled:
      return "enabled";
    case StartupTaskState::EnabledByPolicy:
      return "enabledByPolicy";
  }
  return "unknown";
}

std::wstring Utf8ToWide(const std::string& input) {
  if (input.empty()) return L"";
  int wlen = MultiByteToWideChar(CP_UTF8, 0, input.c_str(),
                                 static_cast<int>(input.size()), nullptr, 0);
  std::wstring result(wlen, L'\0');
  MultiByteToWideChar(CP_UTF8, 0, input.c_str(), static_cast<int>(input.size()),
                      result.data(), wlen);
  return result;
}

}  // namespace

void RegisterStartupTaskChannel(flutter::FlutterViewController* controller) {
  auto channel =
      std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
          controller->engine()->messenger(), kChannelName,
          &flutter::StandardMethodCodec::GetInstance());

  // The channel must outlive the engine. Leak intentionally.
  auto* leaked_channel = channel.release();

  leaked_channel->SetMethodCallHandler(
      [](const flutter::MethodCall<flutter::EncodableValue>& call,
         std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>>
             result) {
        const std::string method = call.method_name();

        std::string task_id;
        if (const auto* args =
                std::get_if<flutter::EncodableMap>(call.arguments())) {
          auto it = args->find(flutter::EncodableValue("taskId"));
          if (it != args->end()) {
            if (const auto* s = std::get_if<std::string>(&it->second)) {
              task_id = *s;
            }
          }
        }

        if (task_id.empty()) {
          result->Error("invalid_args", "taskId required");
          return;
        }

        std::wstring wtask_id = Utf8ToWide(task_id);
        std::shared_ptr<flutter::MethodResult<flutter::EncodableValue>>
            shared_result(result.release());

        std::thread([method, wtask_id, shared_result]() {
          try {
            winrt::init_apartment();
            using namespace winrt::Windows::ApplicationModel;
            auto task = StartupTask::GetAsync(wtask_id).get();
            if (method == "getState") {
              shared_result->Success(
                  flutter::EncodableValue(StateToString(task.State())));
            } else if (method == "enable") {
              auto state = task.RequestEnableAsync().get();
              shared_result->Success(
                  flutter::EncodableValue(StateToString(state)));
            } else if (method == "disable") {
              task.Disable();
              shared_result->Success(
                  flutter::EncodableValue(StateToString(task.State())));
            } else {
              shared_result->NotImplemented();
            }
          } catch (const winrt::hresult_error& e) {
            char code_buf[32];
            snprintf(code_buf, sizeof(code_buf), "0x%08X",
                     static_cast<unsigned int>(e.code()));
            shared_result->Error("winrt_error", code_buf,
                                 flutter::EncodableValue(
                                     winrt::to_string(e.message())));
          } catch (...) {
            shared_result->Error("unknown_error", "Unknown failure");
          }
        }).detach();
      });
}
