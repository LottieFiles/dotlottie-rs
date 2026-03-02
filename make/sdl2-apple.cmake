# SDL2 bundled build pre-cache for Apple targets.
#
# Used via sdl2-sys's SDL2_TOOLCHAIN env var, which sdl2-sys passes as
# -DCMAKE_TOOLCHAIN_FILE to the bundled SDL2 CMake build.
#
# Two problems this file solves:
#
# 1. -Werror=declaration-after-statement
#    SDL2 2.0.x (in sdl2-sys 0.35) adds this flag automatically, but its own
#    HIDAPI (src/hidapi/mac/hid.c) and Cocoa file-ops
#    (src/file/cocoa/SDL_rwopsbundlesupport.m) code violates the rule.
#    Pre-caching these to OFF prevents the check_c_compiler_flag() probes
#    from adding the flag at all.
#
# 2. Desktop OpenGL on iOS
#    When CMAKE_SYSTEM_NAME is not set to "iOS", SDL2 treats the build as
#    macOS and enables SDL_OPENGL (desktop OpenGL).  The iOS SDK has no
#    desktop OpenGL headers so SDL_render_gl.c fails to compile.
#    We detect the iOS SDK via the SDKROOT environment variable (which
#    apple.mk sets to the iPhoneOS or iPhoneSimulator SDK path) and
#    explicitly set CMAKE_SYSTEM_NAME and SDL_OPENGL accordingly.

# ---------------------------------------------------------------------------
# iOS platform detection
# ---------------------------------------------------------------------------
# When SDKROOT contains "iphoneos" or "iphonesimulator", we are cross-compiling
# for iOS / iOS Simulator.  Tell CMake the system name so SDL2's feature
# detection correctly disables desktop-OpenGL, macOS-only frameworks, etc.
if(DEFINED ENV{SDKROOT} AND "$ENV{SDKROOT}" MATCHES "[Ii][Pp]hone")
    set(CMAKE_SYSTEM_NAME   iOS    CACHE STRING "" FORCE)
    # Explicitly off: iOS uses OpenGL ES, not desktop OpenGL.
    set(SDL_OPENGL          OFF    CACHE BOOL   "" FORCE)
endif()

# ---------------------------------------------------------------------------
# Suppress -Werror=declaration-after-statement
# ---------------------------------------------------------------------------
set(HAVE_GCC_WDECLARATION_AFTER_STATEMENT       OFF CACHE BOOL "" FORCE)
set(HAVE_GCC_WERROR_DECLARATION_AFTER_STATEMENT  OFF CACHE BOOL "" FORCE)
