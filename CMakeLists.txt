cmake_minimum_required(VERSION 4.0)

# When packaging rust crate, cwd must not be changed
# so modules are installed into NODE_MODULES_PATH see build.rs
set(CACHE NODE_MODULES_PATH TYPE PATH HELP "Path node_modules directory" VALUE "")
set(CACHE PARENT_PROJECT_PATH TYPE PATH HELP "Path to root tauri project (folder containing src-tauri/)" VALUE "")

if("${NODE_MODULES_PATH}" STREQUAL "")
  find_package(cmake-fetch REQUIRED PATHS node_modules/cmake-fetch)
  find_package(cmake-bare REQUIRED PATHS node_modules/cmake-bare)
  find_package(cmake-bare-bundle REQUIRED PATHS node_modules/cmake-bare-bundle)
else()
  find_package(cmake-npm REQUIRED PATHS "${NODE_MODULES_PATH}/cmake-npm")
  find_package(cmake-fetch REQUIRED PATHS "${NODE_MODULES_PATH}/cmake-fetch")
  find_package(cmake-bare REQUIRED PATHS "${NODE_MODULES_PATH}/cmake-bare")
  find_package(cmake-bare-bundle REQUIRED PATHS "${NODE_MODULES_PATH}/cmake-bare-bundle")
endif()

include(overrides.cmake)

project(tauri_plugin_bare_kit LANGUAGES C CXX VERSION 0.2.2)

if(IOS)
  # bare-make hardcodes min iOS version to 14 which is no longer
  # supported by libc++, treat it as regular warning
  add_compile_options("-Wno-error=#warnings")
endif()

if(WIN32)
  # arm64 windows treats incompatible pointers as error
  # rather than warning, so disable it
  add_compile_options("-Wno-incompatible-pointer-types")
endif()

fetch_package("github:holepunchto/bare-kit@2.3.0")

# Allow installing fetched target
set_target_properties(
  bare_kit
  PROPERTIES
  EXCLUDE_FROM_ALL FALSE
)

if (NOT "${PARENT_PROJECT_PATH}" STREQUAL "")
  if(IOS OR ANDROID)
    # sysroot set, so find_* fails for local files
    # append project root to $PATH to fix that
    list(APPEND CMAKE_FIND_ROOT_PATH ${PARENT_PROJECT_PATH})
  endif()

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
endif()

install(
  TARGETS
    bare_kit
  FRAMEWORK DESTINATION
    "."
  LIBRARY DESTINATION
    "."
  RUNTIME DESTINATION
    "."
  ARCHIVE DESTINATION
    "."
)
