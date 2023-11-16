#!/bin/bash

# Set up initial configurations and paths
PLISTBUDDY_EXEC="/usr/libexec/PlistBuddy"
CRATE_NAME="dotlottie_player"
BINDINGS=./uniffi-bindings

# Create the include directory and set up module map
mkdir -p ./artifacts/include/

cp $BINDINGS/dotlottie_playerFFI.h ./artifacts/include/dotlottie_player.h

cat << EOF > "./artifacts/include/module.modulemap"
framework module DLPlayer {
  umbrella header "dotlottie_player.h"
  export *
  module * { export * }
}
EOF

# Combine libraries using lipo
mkdir -p ./artifacts/macosx_x86_64
# mkdir -p ./artifacts/aarch64-apple-ios

lipo -create \
    "./target/x86_64-apple-darwin/release/libdlplayer.dylib" \
    -o "./artifacts/macosx_x86_64/libdlplayer.dylib"

# lipo -create \
#     "./target/aarch64-apple-ios/release/libdlplayer.dylib" \
#     -o "./artifacts/aarch64-apple-ios/libdlplayer.dylib"

# Prepare the framework for each target
for TARGET_TRIPLE in "macosx_x86_64"; do
    FRAMEWORK_PATH="./artifacts/$TARGET_TRIPLE/DLPlayer.framework"
    
    mkdir -p $FRAMEWORK_PATH/Headers
    mkdir -p $FRAMEWORK_PATH/Modules
    
    mv ./artifacts/$TARGET_TRIPLE/libdlplayer.dylib $FRAMEWORK_PATH/DLPlayer
    cp ./artifacts/include/dotlottie_player.h $FRAMEWORK_PATH/Headers/
    cp ./artifacts/include/module.modulemap $FRAMEWORK_PATH/Modules/

    # Set up the plist for the framework
    $PLISTBUDDY_EXEC -c "Add :CFBundleIdentifier string com.dotlottie.DLPlayer" \
                    -c "Add :CFBundleName string DLPlayer" \
                    -c "Add :CFBundleDisplayName string DLPlayer" \
                    -c "Add :CFBundleVersion string 1.0.0" \
                    -c "Add :CFBundleShortVersionString string 1.0.0" \
                    -c "Add :CFBundlePackageType string FMWK" \
                    -c "Add :CFBundleExecutable string DLPlayer" \
                    -c "Add :MinimumOSVersion string 16.4" \
                    -c "Add :CFBundleSupportedPlatforms array" \
                    $FRAMEWORK_PATH/Info.plist

    case $TARGET_TRIPLE in
        macosx_x86_64)
            $PLISTBUDDY_EXEC -c "Add :CFBundleSupportedPlatforms:1 string MacOSX" $FRAMEWORK_PATH/Info.plist
            ;;
        aarch64-apple-ios)
            $PLISTBUDDY_EXEC -c "Add :CFBundleSupportedPlatforms:0 string iPhoneOS" $FRAMEWORK_PATH/Info.plist
            ;;
        ios-simulator-arm64_x86_64)
            $PLISTBUDDY_EXEC -c "Add :CFBundleSupportedPlatforms:0 string iPhoneOS" \
                             -c "Add :CFBundleSupportedPlatforms:1 string iPhoneSimulator" \
                             $FRAMEWORK_PATH/Info.plist
            ;;
        *)
            ;;
    esac

    install_name_tool -id @rpath/DLPlayer.framework/DLPlayer $FRAMEWORK_PATH/DLPlayer
done

# Create the XCFramework
xcodebuild -create-xcframework \
    -framework "./artifacts/macosx_x86_64/DLPlayer.framework" \
    -output "./artifacts/DLPlayer.xcframework"

echo "Done creating DLPlayer.xcframework!"

BASE_DIR=../ios
rm -rf $BASE_DIR;

# Creating Framework folder
mkdir -p $BASE_DIR/Framework
mkdir -p $BASE_DIR/Bindings

cp -R ./artifacts/DLPlayer.xcframework $BASE_DIR/Framework
cp $BINDINGS/dotlottie_player.swift $BASE_DIR/Bindings
sed -i "" -E 's/[[:<:]]dotlottie_playerFFI[[:>:]]/DLPlayer/g' $BASE_DIR/Bindings/dotlottie_player.swift

#clean up
# rm -rf ./artifacts

echo "Done generating ios Framework"
