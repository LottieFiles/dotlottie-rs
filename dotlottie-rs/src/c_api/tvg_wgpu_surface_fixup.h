#pragma once
/*
 * Force-included into every ThorVG WebGPU compilation unit via build.rs.
 *
 * ThorVG's surfaceConfigure hardcodes WGPUPresentMode_Immediate on Apple
 * targets, but iOS Metal only supports Fifo.  This header redirects every
 * call to wgpuSurfaceConfigure through our shim (_tvg_wgpu_surface_configure_fixup
 * in apple.rs), which corrects the present mode before forwarding to the
 * real wgpu-native implementation.
 */
#if defined(__APPLE__) && !defined(__EMSCRIPTEN__)
#  define wgpuSurfaceConfigure _tvg_wgpu_surface_configure_fixup
#endif
