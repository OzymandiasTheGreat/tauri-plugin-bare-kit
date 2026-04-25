cmake_minimum_required(VERSION 4.0)

find_package(cmake-fetch REQUIRED PATHS node_modules/cmake-fetch)
find_package(cmake-bare REQUIRED PATHS node_modules/cmake-bare)
find_package(cmake-bare-bundle REQUIRED PATHS node_modules/cmake-bare-bundle)

include(overrides.cmake)

project(tauri_plugin_bare_kit LANGUAGES C CXX VERSION 0.1.0)

set(PARENT_PROJECT_PATH "" CACHE PATH "Path to root tauri project (folder containing src-tauri/)")

fetch_package("github:holepunchto/bare-kit@2.0.2")

set_target_properties(
  bare_kit
  PROPERTIES
  EXCLUDE_FROM_ALL FALSE
)

find_file(
  BARE_ENTRY_PATH
  "app.js"
  PATHS
    "${PARENT_PROJECT_PATH}"
  PATH_SUFFIXES
    "bare"
    "bare/dist"
    "bare/src"
    "src-bare"
    "src-bare/dist"
    "src-bare/src"
  REQUIRED
  NO_DEFAULT_PATH
)

generate_builtins(WORKING_DIRECTORY "${PARENT_PROJECT_PATH}")

add_bare_bundle(
  app_bundle
  ENTRY "${BARE_ENTRY_PATH}"
  OUT "${PARENT_PROJECT_PATH}/app.bundle.json"
  BUILTINS "${CMAKE_BINARY_DIR}/builtins.json"
  WORKING_DIRECTORY "${PARENT_PROJECT_PATH}"
)

link_bare_modules(
  bare_kit
  WORKING_DIRECTORY
    "${PARENT_PROJECT_PATH}"
)

install(
  TARGETS
    bare_kit
  FRAMEWORK DESTINATION
    Frameworks
)
