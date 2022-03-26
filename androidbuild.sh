#!/bin/bash

CURRENT=`pwd`

$TARGET=$1

# download our pre-built gmp .a and .so libraries from https://github.com/ezsyfi/gmp github release
mkdir -p $HOME/usr/local
cd $HOME/usr/local

wget https://github.com/ezsyfi/gmp/releases/download/6.2.1/aarch64-linux-android.zip
unzip aarch64-linux-android.zip
rm aarch64-linux-android.zip
wget https://github.com/ezsyfi/gmp/releases/download/6.2.1/x86_64-linux-android.zip
unzip x86_64-linux-android.zip
rm x86_64-linux-android.zip
wget https://github.com/ezsyfi/gmp/releases/download/6.2.1/i686-linux-android.zip
unzip i686-linux-android.zip
rm i686-linux-android.zip
# wget https://github.com/ezsyfi/gmp/releases/download/6.2.1/armv7a-linux-androideabi.zip
# unzip armv7a-linux-androideabi.zip
# rm armv7a-linux-androideabi.zip

cd $CURRENT
mkdir -p target/{aarch64-linux-android,x86_64-linux-android,armv7a-linux-androideabi,i686-linux-android}/release/deps

# place our gmp .a and .so into our cargo build release deps directory
cp -rf $HOME/usr/local/aarch64-linux-android/lib/libgmp* target/aarch64-linux-android/release/deps/
cp -rf $HOME/usr/local/x86_64-linux-android/lib/libgmp* target/x86_64-linux-android/release/deps/
cp -rf $HOME/usr/local/i686-linux-android/lib/libgmp* target/i686-linux-android/release/deps/
# cp -rf $HOME/usr/local/armv7a-linux-androideabi/lib/libgmp* target/armv7-linux-androideabi/release/deps/

# use cargo ndk to build our android libraries
cargo install cargo-ndk

cargo ndk --target=aarch64-linux-android build --lib --release
cargo ndk --target=x86_64-linux-android build --lib --release
cargo ndk --target=i686-linux-android build --lib --release
# cargo ndk --target=armv7-linux-androideabi build --lib --release