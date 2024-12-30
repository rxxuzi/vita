cmake_minimum_required(VERSION 3.27)
project(vita C)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -O2")

add_executable(vita
        src/main.c
        src/options.h
        src/traverse.h
        src/utils.h
        src/traverse.c
        src/utils.c
        src/file.c
        src/file.c
)
