#!/bin/bash

library_name="dotlottie-player"
BASE_PATH="../android"
BINDINGS=./uniffi-bindings
ASSETS=./assets

rm -rf  $BASE_PATH

src=$BASE_PATH/$library_name/src/main/kotlin
jni=$BASE_PATH/$library_name/src/main/jniLibs
package=com/dotlottie/dlplayer

echo "Generating library $library_name"
mkdir -p $BASE_PATH/$library_name
mkdir -p $src/$package
mkdir -p $jni

# Copying .kt file
touch $src/$package/DotLottiePlayer.kt
cp $BINDINGS/$package/*.kt $src/$package/DotLottiePlayer.kt
test -e $src/$package/DotLottiePlayer.kt || exit 1

jna_architectures=(
  "arm64-v8a"
  "armeabi-v7a"
)
target_triples=(
  "aarch64-linux-android"
  "armv7-linux-androideabi"
)


for (( i=0; i<${#jna_architectures[@]}; i++ ));
do
  arch_name=${jna_architectures[$i]}
  target_triple=${target_triples[$i]}

  echo "Extracting for architecture $arch_name"

  mkdir -p $jni/"$arch_name"
  touch $jni/"$arch_name"/libuniffi_dotlottie_player.so
  cp ./target/"$target_triple"/release/*.so $jni/"$arch_name"/libuniffi_dotlottie_player.so
  test -e $jni/"$arch_name"/libuniffi_dotlottie_player.so || exit 1
done

# Initialise Gradle project
cp $ASSETS/android/build.gradle.kts $BASE_PATH/$library_name/
cp $ASSETS/android/consumer-rules.pro $BASE_PATH/$library_name/

# Extract version from Cargo.toml
toml=./Cargo.toml
ret_version=$(grep -m 1 version $toml | awk -F= '{print $2}' | tr -d '" ')
commit_hash=$(git rev-parse --short HEAD)
version="$ret_version-$commit_hash"
touch $BASE_PATH/$library_name/gradle.properties
echo -e "dlplayer-version=$version" >> $BASE_PATH/$library_name/gradle.properties
