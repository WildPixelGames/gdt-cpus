cmake_minimum_required(VERSION 3.15)
project(priority LANGUAGES C)

include(FetchContent)

FetchContent_Declare(
    gdt_cpus_sys_dep
    GIT_REPOSITORY https://github.com/gdt-tools/gdt-cpus-rs.git
    SOURCE_SUBDIR crates/gdt-cpus-sys
)

FetchContent_MakeAvailable(gdt_cpus_sys_dep)

add_executable(priority main.c)

set_property(TARGET priority PROPERTY C_STANDARD 23)

target_link_libraries(priority PRIVATE gdt_cpus_sys)

if(WIN32 AND GDT_CPUS_PREFER_SHARED)
    add_custom_command(TARGET priority POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${gdt_cpus_sys_dep_BINARY_DIR}/rust_build/target/${CARGO_TARGET_DIR_SUFFIX}/${GDT_CPUS_SYS_SHARED_LIB_NAME}"
        $<TARGET_FILE_DIR:priority>
        COMMENT "Copying ${GDT_CPUS_SYS_SHARED_LIB_NAME} to executable directory"
    )
endif()
