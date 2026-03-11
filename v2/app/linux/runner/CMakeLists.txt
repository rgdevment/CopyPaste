cmake_minimum_required(VERSION 3.13)
project(runner LANGUAGES C CXX)

add_executable(${BINARY_NAME}
  "copypaste_linux_shell.c"
  "main.cc"
  "my_application.cc"
  "${FLUTTER_MANAGED_DIR}/generated_plugin_registrant.cc"
)

apply_standard_settings(${BINARY_NAME})
target_compile_options(${BINARY_NAME} PRIVATE -Wno-deprecated-declarations)

add_definitions(-DAPPLICATION_ID="${APPLICATION_ID}")

target_link_libraries(${BINARY_NAME} PRIVATE flutter)
target_link_libraries(${BINARY_NAME} PRIVATE PkgConfig::GTK)

pkg_check_modules(X11 IMPORTED_TARGET x11 xtst)
if(X11_FOUND)
  target_link_libraries(${BINARY_NAME} PRIVATE PkgConfig::X11)
else()
  message(WARNING "X11/XTest not found — hotkey and tray features will be X11-unavailable")
endif()

target_include_directories(${BINARY_NAME} PRIVATE "${CMAKE_SOURCE_DIR}")
