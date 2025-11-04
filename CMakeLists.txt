cmake_minimum_required(VERSION 3.22)

find_package(cmake-fetch REQUIRED PATHS node_modules/cmake-fetch)

project(tauri_plugin_bare_kit LANGUAGES C CXX VERSION 0.0.1)

if(LINUX)
  fetch_package("github:OzymandiasTheGreat/bare-kit#linux")
else()
  fetch_package("github:holepunchto/bare-kit@v1.12.1")
endif()

install(
  DIRECTORY
    $<TARGET_PROPERTY:uv,INTERFACE_INCLUDE_DIRECTORIES>
  TYPE
  INCLUDE
)

install(
  DIRECTORY
    $<TARGET_PROPERTY:bare,INTERFACE_INCLUDE_DIRECTORIES>
  TYPE
  INCLUDE
)

install(
  TARGETS
    bare_kit
  FRAMEWORK DESTINATION
    Frameworks
)
