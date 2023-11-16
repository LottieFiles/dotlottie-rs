rm -rf build
rm -rf final-build

cd ./thorvg/

# meson setup --backend=ninja build -Dloaders="all" -Ddefault_library=static -Dstatic=true -Dsavers="all" -Dbindings="capi" --cross-file ./cross/ios_x86_64.txt

# meson . build -Dloaders="all" -Ddefault_library=static -Dstatic=true -Dsavers="all" -Dbindings="capi" --cross-file ../macos_x86_64.txt

# ~ No cross file aka build for this machine ~
meson . build -Dloaders="all" -Ddefault_library=static -Dstatic=true -Dsavers="all" -Dbindings="capi"
# meson . build -Dloaders="all" -Dsavers="all" -Dbindings="capi"

ninja -C build install

# meson . builddir -Dbindings=capi -Ddefault_library=static && ninja -C builddir install


# mv build ../

# cd ../

# mkdir final-build
# mkdir final-build/include
# mkdir final-build/lib

# cp ./build/src/libthorvg.a ./final-build/lib/
# cp ./build/usr/local/include/* ./final-build/include/