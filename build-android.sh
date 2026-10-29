#!/bin/sh
set -eu
cp /project/Cargo.toml /project/Cargo.lock /root/src/
cp -r /project/src /root/src/
sed -i 's/^version = 4$/version = 3/' /root/src/Cargo.lock
cd /root/src
cargo quad-apk build --release

# Check the generated loader against the actual APK contents, not only Rust compilation.
activity=target/android-artifacts/release/bin/robots/sk/ricochet/robots/MainActivity.java
apk=target/android-artifacts/release/apk/robots.apk
grep -F 'System.loadLibrary("robots")' "$activity"
unzip -Z1 "$apk" | grep -Fx 'lib/arm64-v8a/librobots.so'
echo 'Verified: Java loads robots and the APK contains librobots.so for ARM64.'

# Inspect the packaged binary manifest, not just the source configuration.
"$ANDROID_HOME/build-tools/36.0.0/aapt" dump badging "$apk" > /tmp/robots-badging.txt
grep -Fx "targetSdkVersion:'36'" /tmp/robots-badging.txt
grep -Fx "sdkVersion:'21'" /tmp/robots-badging.txt
"$ANDROID_HOME/build-tools/36.0.0/apksigner" verify "$apk"
echo 'Verified: APK targets Android 16 / API 36 and supports API 21+.'
