cmake_minimum_required(VERSION 4.0)

find_package(cmake-fetch REQUIRED PATHS node_modules/cmake-fetch)

project(tauri_plugin_bare_kit LANGUAGES C CXX VERSION 0.1.0)

fetch_package("github:holepunchto/bare-kit@1.13.3")

set_target_properties(
  bare_kit
  PROPERTIES
  EXCLUDE_FROM_ALL FALSE
)

install(
  TARGETS
    bare_kit
  FRAMEWORK DESTINATION
    Frameworks
)
