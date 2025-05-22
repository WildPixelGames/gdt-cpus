cmake_minimum_required(VERSION 3.15)
project(basic_info LANGUAGES C)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

include(FetchContent)

FetchContent_Declare(
    gdt_cpus_sys_dep
    GIT_REPOSITORY https://github.com/WildPixelGames/gdt-cpus.git
    SOURCE_SUBDIR crates/gdt-cpus-sys
)

FetchContent_MakeAvailable(gdt_cpus_sys_dep)

add_executable(basic_info main.c)

set_property(TARGET basic_info PROPERTY C_STANDARD 23)

target_link_libraries(basic_info PRIVATE gdt_cpus_sys)

if(WIN32 AND GDT_CPUS_PREFER_SHARED)
    add_custom_command(TARGET basic_info POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${gdt_cpus_sys_dep_BINARY_DIR}/rust_build/target/${CARGO_TARGET_DIR_SUFFIX}/${GDT_CPUS_SYS_SHARED_LIB_NAME}" # Adjust this path based on actual output path of DLL
        $<TARGET_FILE_DIR:basic_info>
        COMMENT "Copying ${GDT_CPUS_SYS_SHARED_LIB_NAME} to executable directory"
    )
endif()
