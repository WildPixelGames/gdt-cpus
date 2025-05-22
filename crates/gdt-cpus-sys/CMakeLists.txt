cmake_minimum_required(VERSION 4.0)
project(gdt-cpus-sys LANGUAGES NONE)

include(FetchContent)
FetchContent_Declare(
    Corrosion_dep
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG        v0.5.2
)
FetchContent_MakeAvailable(Corrosion_dep)

corrosion_import_crate(MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml)

set(GDT_CPUS_SYS_INCLUDE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/include)

if(TARGET gdt_cpus_sys)
    message(STATUS "Configuring INTERFACE target 'gdt_cpus_sys' created by Corrosion")

    target_include_directories(gdt_cpus_sys INTERFACE
        $<BUILD_INTERFACE:${GDT_CPUS_SYS_INCLUDE_DIR}>
        # $<INSTALL_INTERFACE:include>
    )
else()
    message(FATAL_ERROR "Corrosion did not create the expected target 'gdt_cpus_sys'. "
                        "Check Corrosion output or documentation for default target naming conventions. "
                        "Ensure the crate name in Cargo.toml matches.")
endif()

message(STATUS "Minimal gdt-cpus-sys CMake configuration complete. Consumer should link 'gdt_cpus_sys'.")
