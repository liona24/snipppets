cmake_minimum_required (VERSION 3.8)

project ("at_your_service")

add_executable (at_your_service "at_your_service.c")

if (MSVC)
    add_compile_options(/MT /GS- /O1 /Oi)
else()
    message(FATAL_ERROR "Go figure compiler out yourself :(")
endif()

target_link_libraries(at_your_service wtsapi32.lib)
