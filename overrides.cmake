function(add_bare_module result)
  bare_module_target("." target NAME name)

  if(NOT TARGET ${target})
    add_library(${target} OBJECT)

    set_target_properties(
      ${target}
      PROPERTIES
      C_STANDARD 11
      CXX_STANDARD 20
      POSITION_INDEPENDENT_CODE ON
    )

    target_include_directories(
      ${target}
      PRIVATE
        $<TARGET_PROPERTY:bare,INTERFACE_INCLUDE_DIRECTORIES>
    )
  endif()

  set(${result} ${target})

  return(PROPAGATE ${result})
endfunction()

function(add_napi_module result)
  napi_module_target("." target NAME name)

  if(NOT TARGET ${target})
    add_library(${target} OBJECT)

    set_target_properties(
      ${target}
      PROPERTIES
      C_STANDARD 11
      CXX_STANDARD 20
      POSITION_INDEPENDENT_CODE ON
    )

    target_include_directories(
      ${target}
      PRIVATE
        $<TARGET_PROPERTY:bare,INTERFACE_INCLUDE_DIRECTORIES>
    )
  endif()

  set(${result} ${target})

  return(PROPAGATE ${result})
endfunction()

function(link_bare_modules receiver)
  set(option_keywords
    SHARED
  )

  set(one_value_keywords
    WORKING_DIRECTORY
  )

  set(multi_value_keywords
    EXCLUDE
  )

  cmake_parse_arguments(
    PARSE_ARGV 1 ARGV "${option_keywords}" "${one_value_keywords}" "${multi_value_keywords}"
  )

  if(ARGV_WORKING_DIRECTORY)
    cmake_path(ABSOLUTE_PATH ARGV_WORKING_DIRECTORY BASE_DIRECTORY "${CMAKE_CURRENT_LIST_DIR}" NORMALIZE)
  else()
    set(ARGV_WORKING_DIRECTORY "${CMAKE_CURRENT_LIST_DIR}")
  endif()

  if(ARGV_SHARED)
    set(SHARED SHARED)
  else()
    set(SHARED)
  endif()

  list_node_modules(
    packages
    WORKING_DIRECTORY "${ARGV_WORKING_DIRECTORY}"
  )

  foreach(base ${packages})
    cmake_path(APPEND base "package.json" OUTPUT_VARIABLE package_path)

    file(READ "${package_path}" package)

    string(JSON name ERROR_VARIABLE error GET "${package}" "name")

    if("${name}" IN_LIST ARGV_EXCLUDE)
      continue()
    endif()

    if("${name}" IN_LIST SEEN)
      continue()
    endif()

    list(APPEND SEEN "${name}")

    string(JSON addon ERROR_VARIABLE error GET "${package}" "addon")

    if(addon)
      link_bare_module(
        ${receiver}
        ${base}
        ${SHARED}
        WORKING_DIRECTORY "${ARGV_WORKING_DIRECTORY}"
      )
    endif()
  endforeach()
endfunction()

function(generate_builtins)
  set(one_value_keywords
    WORKING_DIRECTORY
  )

  cmake_parse_arguments(PARSE_ARGV 0 ARGV "" "${one_value_keywords}" "")

  if(ARGV_WORKING_DIRECTORY)
    cmake_path(ABSOLUTE_PATH ARGV_WORKING_DIRECTORY BASE_DIRECTORY "${CMAKE_CURRENT_LIST_DIR}" NORMALIZE)
  else()
    set(ARGV_WORKING_DIRECTORY "${CMAKE_CURRENT_LIST_DIR}")
  endif()

  list_node_modules(
    packages
    WORKING_DIRECTORY "${ARGV_WORKING_DIRECTORY}"
  )

  set(builtins "[]")

  foreach(base ${packages})
    cmake_path(APPEND base "package.json" OUTPUT_VARIABLE package_path)

    file(READ "${package_path}" package)

    string(JSON name ERROR_VARIABLE error GET "${package}" "name")

    string(JSON version ERROR_VARIABLE error GET "${package}" "version")

    if("${name}@${version}" IN_LIST SEEN)
      continue()
    endif()

    list(APPEND SEEN "${name}@${version}")

    string(JSON addon ERROR_VARIABLE error GET "${package}" "addon")

    if(addon)
      set(entry "{\"addon\": \"${name}@${version}\"}")

      string(JSON index ERROR_VARIABLE error LENGTH "${builtins}")

      string(JSON builtins ERROR_VARIABLE error SET "${builtins}" "${index}" "${entry}")
    endif()
  endforeach()

  file(WRITE "${CMAKE_BINARY_DIR}/builtins.json" "${builtins}")
endfunction()
