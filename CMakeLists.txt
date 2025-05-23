cmake_minimum_required(VERSION 3.12 FATAL_ERROR)
cmake_policy(VERSION 3.12)

project(Bear
        VERSION 3.1.6
        DESCRIPTION "Bear is a tool to generate compilation database for clang tooling."
        LANGUAGES C CXX
        )

# Avoid warning about DOWNLOAD_EXTRACT_TIMESTAMP in CMake 3.24:
if (CMAKE_VERSION VERSION_GREATER_EQUAL "3.24.0")
    cmake_policy(SET CMP0135 NEW)
endif()

option(ENABLE_UNIT_TESTS "Build and run unit test for this project" ON)
option(ENABLE_FUNC_TESTS "Build and run functional test for this project" ON)
option(ENABLE_MULTILIB "Enable to build with multilib support" OFF)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(default_build_type "Release")
if (NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
    message(STATUS "Setting build type to '${default_build_type}' as none was specified.")
    set(CMAKE_BUILD_TYPE "${default_build_type}" CACHE
            STRING "Choose the type of build." FORCE)
    # Set the possible values of build type for cmake-gui
    set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS
            "Debug" "Release" "MinSizeRel" "RelWithDebInfo")
endif ()

# Superbuild variables for sub projects
include(ExternalProject)
set_property(DIRECTORY PROPERTY EP_BASE ${CMAKE_BINARY_DIR}/subprojects)
set(STAGED_INSTALL_PREFIX ${CMAKE_BINARY_DIR}/stage)
set(DEPENDENCIES_INSTALL_PREFIX ${CMAKE_BINARY_DIR}/subprojects/Install)
include(GNUInstallDirs)

# Verify or install dependencies
add_subdirectory(third_party)

unset(CMAKE_CACHE_ARGS_EXTRA)
if (CMAKE_PROJECT_INCLUDE AND NOT CMAKE_PROJECT_INCLUDE STREQUAL "")
    set(CMAKE_CACHE_ARGS_EXTRA "-DCMAKE_PROJECT_INCLUDE:PATH=${CMAKE_PROJECT_INCLUDE}")
endif()

# Build the project itself
ExternalProject_Add(BearSource
        SOURCE_DIR
            "${CMAKE_CURRENT_SOURCE_DIR}/source"
        DEPENDS
            nlohmann_json_dependency
            fmt_dependency
            spdlog_dependency
            grpc_dependency
            googletest_dependency
        CMAKE_ARGS
            -DENABLE_UNIT_TESTS:BOOL=${ENABLE_UNIT_TESTS}
            -DENABLE_MULTILIB:BOOL=${ENABLE_MULTILIB}
            -DPKG_CONFIG_EXECUTABLE:PATH=${PKG_CONFIG_EXECUTABLE}
        CMAKE_CACHE_ARGS
            -DCMAKE_PROJECT_VERSION:STRING=${CMAKE_PROJECT_VERSION}
            -DCMAKE_BUILD_TYPE:STRING=${CMAKE_BUILD_TYPE}
            -DCMAKE_TOOLCHAIN_FILE:PATH=${CMAKE_TOOLCHAIN_FILE}
            -DCMAKE_FIND_ROOT_PATH:PATH=${CMAKE_FIND_ROOT_PATH}
            -DCMAKE_IGNORE_PATH:PATH=${CMAKE_IGNORE_PATH}
            -DCMAKE_SYSROOT:PATH=${CMAKE_SYSROOT}
            -DCMAKE_C_COMPILER:STRING=${CMAKE_C_COMPILER}
            -DCMAKE_C_COMPILER_TARGET:STRING=${CMAKE_C_COMPILER_TARGET}
            -DCMAKE_C_FLAGS:STRING=${CMAKE_C_FLAGS}
            -DCMAKE_CXX_COMPILER:STRING=${CMAKE_CXX_COMPILER}
            -DCMAKE_CXX_COMPILER_TARGET:STRING=${CMAKE_CXX_COMPILER_TARGET}
            -DCMAKE_CXX_STANDARD:STRING=${CMAKE_CXX_STANDARD}
            -DCMAKE_CXX_STANDARD_REQUIRED:BOOL=${CMAKE_CXX_STANDARD_REQUIRED}
            -DCMAKE_CXX_EXTENSIONS:BOOL=${CMAKE_CXX_EXTENSIONS}
            -DCMAKE_CXX_FLAGS:STRING=${CMAKE_CXX_FLAGS}
            -DCMAKE_PREFIX_PATH:PATH=${CMAKE_PREFIX_PATH}
            -DCMAKE_INSTALL_PREFIX:PATH=${STAGED_INSTALL_PREFIX}
            -DCMAKE_INSTALL_LIBDIR:PATH=${CMAKE_INSTALL_LIBDIR}
            -DCMAKE_EXE_LINKER_FLAGS:STRING=${CMAKE_EXE_LINKER_FLAGS}
            -DCMAKE_SHARED_LINKER_FLAGS:STRING=${CMAKE_SHARED_LINKER_FLAGS}
            -DCMAKE_MODULE_LINKER_FLAGS:STRING=${CMAKE_MODULE_LINKER_FLAGS}
            -DROOT_INSTALL_PREFIX:PATH=${CMAKE_INSTALL_PREFIX}
            ${CMAKE_CACHE_ARGS_EXTRA}
        BUILD_ALWAYS
            1
        TEST_BEFORE_INSTALL
            1
        TEST_COMMAND
            ctest # or `ctest -T memcheck`
        )

# Run the functional tests
if (ENABLE_FUNC_TESTS)
    ExternalProject_Add(BearTest
            SOURCE_DIR
                "${CMAKE_CURRENT_SOURCE_DIR}/test"
            DEPENDS
                BearSource
            CMAKE_CACHE_ARGS
                -DCMAKE_INSTALL_LIBDIR:PATH=${CMAKE_INSTALL_LIBDIR}
                -DCMAKE_INSTALL_BINDIR:PATH=${CMAKE_INSTALL_BINDIR}
                -DSTAGED_INSTALL_PREFIX:PATH=${STAGED_INSTALL_PREFIX}
            TEST_BEFORE_INSTALL
                1
            INSTALL_COMMAND
                ""
            TEST_COMMAND
                ctest --verbose
            )
endif ()

# Install the project artifacts from the staged directory
include(GNUInstallDirs)
install(DIRECTORY
            ${STAGED_INSTALL_PREFIX}/
        DESTINATION
            .
        USE_SOURCE_PERMISSIONS
)
install(FILES
            COPYING README.md INSTALL.md CONTRIBUTING.md CODE_OF_CONDUCT.md
        DESTINATION
            ${CMAKE_INSTALL_DOCDIR}
)

# Set up package from this project
set(CPACK_PACKAGE_NAME "bear")
set(CPACK_PACKAGE_CONTACT "László Nagy")
set(CPACK_PACKAGE_VENDOR ${CPACK_PACKAGE_CONTACT})
set(CPACK_PACKAGE_VERSION ${CMAKE_PROJECT_VERSION})
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "BuildEAR")
set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_CURRENT_SOURCE_DIR}/COPYING")
set(CPACK_RESOURCE_FILE_README "${CMAKE_CURRENT_SOURCE_DIR}/README.md")
set(CPACK_PACKAGING_INSTALL_PREFIX ${CMAKE_INSTALL_PREFIX})
set(CPACK_RPM_PACKAGE_RELEASE "1%{?dist}")
set(CPACK_RPM_PACKAGE_LICENSE "GPLv3")
set(CPACK_RPM_PACKAGE_GROUP "Development/Tools")
set(CPACK_RPM_PACKAGE_URL "http://github.com/rizsotto/Bear")
set(CPACK_RPM_PACKAGE_DESCRIPTION "Bear is a tool to generate compilation database for clang tooling.")
set(CPACK_RPM_EXCLUDE_FROM_AUTO_FILELIST_ADDITION
        "${CMAKE_INSTALL_MANDIR}"
        "${CMAKE_INSTALL_MANDIR}/man1")
set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
include(CPack)
