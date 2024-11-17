#![feature(panic_payload_as_str)]

mod event_handler;

use evdev::KeyCode;
use jni::objects::{JClass, JObject, JStaticMethodID, JValueGen};
use jni::signature::{Primitive, ReturnType};
use jni::sys::{jint, JNI_VERSION_1_6};
use jni::{JNIEnv, JavaVM};
use std::os::raw::{c_char, c_int};
use tokio::time::Duration;

#[link(name = "log")]
extern "C" {
    pub fn __android_log_print(prio: c_int, tag: *const c_char, fmt: *const c_char, ...) -> c_int;
}

#[macro_export]
macro_rules! cprintf {
    ($s:literal) => {{
        unsafe {
            let __s = concat!($s, '\0').as_ptr() as *const _;
            libc::printf(__s);
            $crate::__android_log_print(
                3,
                "volev\0".as_ptr() as *const _,
                __s,
            );
        };
    }};
    ($s:literal, $($args:expr),*) => {{
        unsafe {
            let __s = concat!($s, '\0').as_ptr() as *const _;
            libc::printf(__s, $($args),*);
            $crate::__android_log_print(
                3,
                "volev\0".as_ptr() as *const _,
                __s,
                $($args),*
            );
        };
    }};
}

struct JavaMethod<'a> {
    class: &'a JClass<'a>,

    id: JStaticMethodID,
    ret: ReturnType,
}
impl<'a> JavaMethod<'a> {
    unsafe fn new(
        env: &mut JNIEnv<'_>,
        class: &'a JClass<'a>,
        ret: ReturnType,
        name: &'static str,
        sig: &'static str,
    ) -> Self {
        let id = env
            .get_static_method_id(class, name, sig)
            .unwrap_unchecked();
        Self { class, id, ret }
    }

    unsafe fn call(&self, env: &mut JNIEnv<'a>) -> JValueGen<JObject<'a>> {
        env.call_static_method_unchecked(self.class, self.id, self.ret.clone(), &[])
            .unwrap_unchecked()
    }
}

#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _: *mut ()) -> jint {
    std::panic::set_hook(Box::new(|p| {
        cprintf!("panicked\n");
        if let Some(loc) = p.location() {
            cprintf!(
                "at %.*s:%d:%d\n",
                loc.file().bytes().len(),
                loc.file().as_ptr() as *const _,
                loc.line(),
                loc.column()
            );
        }
        if let Some(pstr) = p.payload_as_str() {
            cprintf!("%.*s\n", pstr.len(), pstr.as_ptr() as *const _);
        }
    }));

    cprintf!("Loaded\n");
    let mut env = vm.get_env().unwrap();
    let class = env.find_class("com/jhc/Main").unwrap();
    std::mem::forget(env.new_global_ref(&class).unwrap());

    let send_media_next = unsafe {
        JavaMethod::new(
            &mut env,
            &class,
            ReturnType::Primitive(Primitive::Void),
            "sendMediaNextEvent",
            "()V",
        )
    };
    let send_media_prev = unsafe {
        JavaMethod::new(
            &mut env,
            &class,
            ReturnType::Primitive(Primitive::Void),
            "sendMediaPrevEvent",
            "()V",
        )
    };
    let is_interactive = unsafe {
        JavaMethod::new(
            &mut env,
            &class,
            ReturnType::Primitive(Primitive::Boolean),
            "isInteractive",
            "()Z",
        )
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(async {
        let (d, u) = event_handler::get_vol_dev().unwrap();

        let mut voldown_stream = d.into_event_stream().unwrap();
        let mut volup_stream = u.into_event_stream().unwrap();

        let mut voldown_hold = false;
        let mut volup_hold = false;
        macro_rules! handle_event {
            ($ev:expr, $key_code:expr, $vol_state:expr, $ev_stream:expr) => {
                let ev = $ev.unwrap();
                let is_inter = unsafe { is_interactive.call(&mut env).z().unwrap_unchecked() };
                if is_inter || ev.code() != $key_code {
                    continue;
                }
                $vol_state = ev.value() == 1;
                if $vol_state {
                    // let _ = $ev_stream
                    //     .device_mut()
                    //     .send_events(&[InputEvent::new(1, $key_code, 0), InputEvent::new(0, 0, 0)]);
                    // let ev = $ev_stream.next_event().await.unwrap();
                    // println!("-> {:x} {}", ev.code(), ev.value());
                    // let ev = $ev_stream.next_event().await.unwrap();
                    // println!("-> {:x} {}", ev.code(), ev.value());
                    cprintf!("Grabbed\n");
                    $ev_stream.device_mut().grab().unwrap();
                } else {
                    cprintf!("Ungrabbed\n");
                    $ev_stream.device_mut().ungrab().unwrap();
                }
            };
        }

        cprintf!("Start event loop\n");
        loop {
            tokio::select! {
                ev = voldown_stream.next_event() => { handle_event!(ev, KeyCode::KEY_VOLUMEDOWN.0, voldown_hold, voldown_stream); },
                _ = tokio::time::sleep(Duration::from_millis(700)), if voldown_hold  => {
                    voldown_hold = false;
                    unsafe { send_media_prev.call(&mut env); }
                    cprintf!("hold down -> KEYCODE_MEDIA_PREVIOUS\n");
                }
                ev = volup_stream.next_event() => { handle_event!(ev, KeyCode::KEY_VOLUMEUP.0, volup_hold, volup_stream); },
                _ = tokio::time::sleep(Duration::from_millis(700)), if volup_hold  => {
                    volup_hold = false;
                    unsafe { send_media_next.call(&mut env); }
                    cprintf!("hold up -> KEYCODE_MEDIA_NEXT\n");
                }
            }
        }
    });

    JNI_VERSION_1_6
}
