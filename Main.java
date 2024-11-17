package com.jhc;

import android.os.Build;
import android.os.PowerManager;
import android.hardware.input.InputManager;
import android.view.KeyEvent;
import android.content.Context;
import android.view.InputDevice;

// import android.media.AudioManager;
// import android.media.session.MediaSessionManager;
// import android.os.Handler;
import static android.view.KeyEvent.FLAG_FROM_SYSTEM;
import static android.view.KeyEvent.KEYCODE_MEDIA_NEXT;
import static android.view.KeyEvent.KEYCODE_MEDIA_PREVIOUS;

public class Main {
    private final static InputManager inputManager;
    private final static PowerManager powerManager;

    static {
        android.os.Looper.prepareMainLooper();
        Context context = android.app.ActivityThread.systemMain().getSystemContext();
        inputManager = (InputManager) context.getSystemService(InputManager.class);
        powerManager = (PowerManager) context.getSystemService(PowerManager.class);
    }

    public static void main(String[] args) {
        System.out.println(Build.MANUFACTURER + " " + Build.MODEL);
        System.loadLibrary("volev");
    }

    public static boolean isInteractive() {
        return powerManager.isInteractive();
    }

    public static void sendMediaPrevEvent() {
        try {
            sendEvent(new KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_MEDIA_PREVIOUS));
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public static void sendMediaNextEvent() {
        try {
            sendEvent(new KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_MEDIA_NEXT));
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private static void sendEvent(KeyEvent event) {
        long eventTime = android.os.SystemClock.uptimeMillis();
        event.setTime(eventTime, eventTime);
        event.setSource(InputDevice.SOURCE_KEYBOARD);
        // event.setFlags(event.getFlags() | KeyEvent.FLAG_FROM_SYSTEM);
        if (!inputManager.injectInputEvent(event, 2)) {
            System.out.println("sendEvent failed");
        }
    }

    // public static Class<?> getInputManagerClass() {
    //     try {
    //         return Class.forName("android.hardware.input.InputManagerGlobal");
    //     } catch (ClassNotFoundException e) {
    //         return android.hardware.input.InputManager.class;
    //     }
    // }
}