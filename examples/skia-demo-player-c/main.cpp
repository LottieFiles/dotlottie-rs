#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <libgen.h> // For dirname
#include <limits.h> // For PATH_MAX
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <unistd.h> // For readlink
                    //
#include "include/codec/SkCodec.h"
#include "include/core/SkAlphaType.h"
#include "include/core/SkBitmap.h"
#include "include/core/SkCanvas.h"
#include "include/core/SkColorType.h"
#include "include/core/SkData.h"
#include "include/core/SkGraphics.h"
#include "include/core/SkImageInfo.h"
#include "include/core/SkSurface.h"

extern "C" {
#include "../../dotlottie-ffi/bindings.h"
}

#define WIDTH 1000
#define HEIGHT 1000

void usage(char *app) {
  fprintf(stderr, "usage: %s <animation-file>\n", app);
  exit(1);
}

int main(int argc, char **argv) {
  const char *animation_path;
  Display *display;
  Window window;
  XEvent event;
  int screen;
  GC gc;
  XImage *image;
  const uint32_t *buffer;
  int len;
  KeySym key;
  char key_pressed[255];
  int ret;
  int ready;
  float current_frame;

  // Ensure a file path has been provided
  if (argc != 2) {
    usage(argv[0]);
  }
  // Ensure the file path is readable
  animation_path = argv[1];
  ret = access(animation_path, R_OK);
  if (ret != 0) {
    fprintf(stderr, "Invalid animation path\n\n");
    usage(argv[0]);
  }

  // Setup dotlottie config
  DotLottieConfig config;
  dotlottie_init_config(&config);
  config.loop_animation = true;
  config.background_color = 0xffffffff;
  config.layout.fit = Void;
  config.layout.align_x = 1.0;
  config.layout.align_y = 0.5;
  strcpy((char *)config.marker.value, "feather");

  // Setu dotlottie player
  DotLottiePlayer *player = dotlottie_new_player(&config);
  if (!player) {
    fprintf(stderr, "Could not create dotlottie player\n");
    return 1;
  }
  // Load the animation file
  ret = dotlottie_load_animation_path(player, animation_path, WIDTH, HEIGHT);
  if (ret != 0) {
    fprintf(stderr, "Could not load dotlottie animation file\n");
    return 1;
  }
  // Get direct access to the underlying buffer
  ret = dotlottie_buffer_ptr(player, &buffer);
  if (ret != 0) {
    fprintf(stderr, "Could not access underlying dotlottie buffer\n");
    return 1;
  }

  // Setup skia buffer
  SkImageInfo info = SkImageInfo::MakeN32Premul(WIDTH, HEIGHT);
  size_t rowBytes = info.minRowBytes();
  size_t size = info.computeByteSize(rowBytes);
  void *pixels = malloc(size);
  // Setup Skia canvas
  std::unique_ptr<SkCanvas> canvas = SkCanvas::MakeRasterDirect(info, pixels, rowBytes);

  // Setup the display
  display = XOpenDisplay(NULL);
  if (display == NULL) {
    fprintf(stderr, "Cannot open X display\n");
    return 1;
  }
  screen = DefaultScreen(display);
  // Setup a window & drawing context
  window = XCreateSimpleWindow(display, RootWindow(display, screen), 10, 10, WIDTH, HEIGHT, 1,
                               BlackPixel(display, screen), WhitePixel(display, screen));
  XSelectInput(display, window, ExposureMask | KeyPressMask);
  gc = XCreateGC(display, window, 0, NULL);
  XMapWindow(display, window);
  // Create an image over the dotlottie buffer
  image = XCreateImage(display, DefaultVisual(display, screen), DefaultDepth(display, screen),
                       ZPixmap, 0, (char *)pixels, WIDTH, HEIGHT, 32, 0);

  ready = 0;
  current_frame = 0;
  while (1) {
    // Process X events
    while (XPending(display)) {
      XNextEvent(display, &event);
      if (event.type == Expose) {
        ready = 1;
      } else if (event.type == KeyPress) {
        // Handle keypresses
        len = XLookupString(&event.xkey, key_pressed, 255, &key, 0);
        if (len == 1) {
          switch (key_pressed[0]) {
          case 'p':
            ret = dotlottie_play(player);
            if (ret != 0) {
              fprintf(stderr, "Could not start dotlottie player\n");
            }
            break;
          case 's':
            ret = dotlottie_stop(player);
            if (ret != 0) {
              fprintf(stderr, "Could not stop dotlottie player\n");
            }
            break;
          case 'q':
            goto quit;
          }
        }
      }
    }

    if (ready == 1) {
      float next_frame = 0;
      dotlottie_request_frame(player, &next_frame);
      if (next_frame != current_frame) {
        // Process the next frame
        dotlottie_set_frame(player, next_frame);
        dotlottie_render(player);
        // Use skia to render an image
        SkImageInfo imageInfo =
            SkImageInfo::Make(WIDTH, HEIGHT, kBGRA_8888_SkColorType, kPremul_SkAlphaType);
        sk_sp<SkData> imageData = SkData::MakeWithoutCopy(buffer, WIDTH * HEIGHT * 4);
        sk_sp<SkImage> bitmapImage = SkImages::RasterFromData(imageInfo, imageData, WIDTH * 4);
        // Draw the image
        SkRect src = SkRect::MakeWH(bitmapImage->width(), bitmapImage->height());
        SkRect dst = SkRect::MakeWH(WIDTH, HEIGHT);
        canvas->drawImageRect(bitmapImage, src, dst, SkSamplingOptions(), nullptr,
                              SkCanvas::kStrict_SrcRectConstraint);
        // Render the image in the window
        XPutImage(display, window, gc, image, 0, 0, 0, 0, WIDTH, HEIGHT);
        current_frame = next_frame;
      }
    }
  }

quit:
  // Clean up
  XDestroyImage(image); // This also frees the buffer
  XFreeGC(display, gc);
  XCloseDisplay(display);

  return ret;
}
