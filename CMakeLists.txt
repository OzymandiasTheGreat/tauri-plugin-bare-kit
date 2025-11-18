cmake_minimum_required(VERSION 3.22)

find_package(cmake-fetch REQUIRED PATHS node_modules/cmake-fetch)

project(tauri_plugin_bare_kit LANGUAGES C CXX VERSION 0.1.0)

fetch_package("github:holepunchto/bare-kit@v1.12.3")

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
