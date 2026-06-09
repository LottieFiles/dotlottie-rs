#pragma once
/*
 * Force-included into every ThorVG WebGPU compilation unit via build.rs.
 *
 * wgpu-native's Metal backend creates an "(wgpu internal) Signal"
 * MTLCommandBuffer on every wgpuQueueSubmit call.  This Signal holds a
 * Metal retain on a "(wgpu internal) Staging" buffer.  The Signal is
 * autoreleased, so it stays alive until the current thread's autorelease
 * pool drains.
 *
 * Metal Debug validates that no command buffer still references a buffer
 * when that buffer is destroyed (-[MTLDebugDevice
 * notifyExternalReferencesNonZeroOnDealloc]).  ThorVG makes multiple
 * wgpuQueueSubmit calls per frame (render passes + blit in sync()), and
 * wgpu recycles the previous Staging buffer at each new submit.  If the
 * previous Signal ObjC object is still alive in the autorelease pool at
 * that point, the assertion fires.
 *
 * Fix: intercept every wgpuQueueSubmit in ThorVG's code and push/pop an
 * autorelease pool around the real call.  The pop immediately releases the
 * new Signal's autorelease hold.  Metal then holds the only remaining
 * retain; after GPU completion Metal releases it and Signal is fully
 * dealloc'd — well before the next submit recycles the Staging buffer.
 *
 * The shim (_tvg_wgpu_queue_submit_shim) is implemented in apple.rs.
 */
#if defined(__APPLE__) && !defined(__EMSCRIPTEN__)
#  define wgpuQueueSubmit _tvg_wgpu_queue_submit_shim
#endif
