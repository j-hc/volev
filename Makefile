BUILD_DIR=build

JAVAC_OPTS=-source 1.8 -target 1.8 -cp .:android-35.jar

JARFILE=main.jar

$(BUILD_DIR)/$(JARFILE): Main.java
	mkdir -p $(BUILD_DIR)	
	javac $(JAVAC_OPTS) -d $(BUILD_DIR)/classes Main.java
	$(ANDROID_HOME)/build-tools/35.0.0/d8 --min-api 29 --output $(BUILD_DIR)/$(JARFILE) ./$(BUILD_DIR)/classes/com/jhc/*.class # with d8
	# $(ANDROID_HOME)/build-tools/30.0.3/dx --output=$(BUILD_DIR)/$(JARFILE) --dex ./$(BUILD_DIR)/classes # with dx

$(BUILD_DIR)/libvolev.so: volev/src/* volev/Cargo.toml
	mkdir -p $(BUILD_DIR)
	cd volev; ./b.sh
	cp -f volev/target/aarch64-linux-android/release-pr/libvolev.so $(BUILD_DIR)/libvolev.so

.PHONY : clean deploy

all: $(BUILD_DIR)/$(JARFILE) $(BUILD_DIR)/libvolev.so

clean:
	test -d $(BUILD_DIR) && rm -rf $(BUILD_DIR)
	# test -d volev/target && rm -rf volev/target

deploy:
	adb push build/main.jar build/libvolev.so volev.sh /data/local/tmp/
	adb shell chmod a+x /data/local/tmp/volev.sh


# ANDROID_NDK_STANDALONE=$ANDROID_HOME/ndk/26.1.10909125/toolchains/llvm/prebuilt/linux-x86_64

# $(BUILD_DIR)/libvolev.so : sample-jni.c
# 	test -d $(BUILD_DIR) || mkdir $(BUILD_DIR)
# 	$(ANDROID_NDK_STANDALONE)/bin/clang \
# 		$(TARGET) --gcc-toolchain=$(ANDROID_NDK_STANDALONE) \
# 		--sysroot $(ANDROID_NDK_STANDALONE)/sysroot \
# 		-L$(ANDROID_NDK_STANDALONE)/sysroot/usr/lib \
# 		-shared -g -DANDROID -fdata-sections -ffunction-sections -funwind-tables \
# 		-fstack-protector-strong -no-canonical-prefixes -fno-addrsig -fPIC \
# 		$(CFLAGS) -Wl,--exclude-libs,libgcc.a -Wl,--exclude-libs,libatomic.a \
# 		-Wl,--build-id -Wl,--warn-shared-textrel \
# 		-Wl,--no-undefined -Wl,--as-needed \
# 		$(LINKFLAGS) -Wl,-llog \
# 		-Wl,-soname,libvolev.so \
# 		-o $(BUILD_DIR)/libvolev.so sample-jni.c 