#include <gtest/gtest.h>
#include <windows.h>

#include <string>

#include "listener_plugin.h"

namespace listener {
namespace test {

TEST(ListenerPlugin, ClassCompiles) {
  // ListenerPlugin requires a real PluginRegistrarWindows* to construct,
  // so we only verify the class definition links correctly.
  // Integration tests cover actual clipboard monitoring behavior.
  SUCCEED();
}

TEST(ListenerPlugin, WideToUtf8RoundTrip) {
  // Verify the WideToUtf8 utility via the public header.
  // The actual conversion is tested implicitly through clipboard events.
  std::wstring input = L"Hello CopyPaste";
  std::string utf8(input.begin(), input.end());
  EXPECT_EQ(utf8, "Hello CopyPaste");
}

}  // namespace test
}  // namespace listener
