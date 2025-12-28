# export ANDROID_SDK_ROOT=.../Android/Sdk
# export ANDROID_NDK_ROOT=.../Android/Sdk/ndk/29.0.14206865

cargo ndk -t arm64-v8a -o ./java/app/src/main/jniLibs build

cd java
./gradlew build
cd ..
