//! iOS-only workaround: stretches the `WKWebView` to fill the whole UIWindow
//! instead of being constrained to `safeAreaLayoutGuide.layoutFrame`.
//!
//! Background: Tauri 2 / wry creates the WKWebView with its frame pinned to
//! the root view controller's safe-area subset. The web content then reads
//! `window.innerHeight` = (screen height − top inset − bottom inset), but
//! the CSS `env(safe-area-inset-bottom)` keeps returning the device inset
//! (≈34pt on devices with a home indicator), so any `position: fixed; bottom: 0`
//! element sits ~34..81pt **above** the physical bottom of the screen —
//! the gap is the unreachable native background.
//!
//! Fix: after the webview is built we resize it to `superview.bounds` and
//! disable scrollview content-inset auto-adjust. We re-apply the fix a few
//! times because iOS may relayout the view shortly after launch (storyboard
//! flush, status bar, etc.).
//!
//! All Objective-C calls go through `msg_send!` on a raw pointer that Tauri
//! hands us via `WebviewWindow::with_webview` — no extra Swift/ObjC files.
#![cfg(target_os = "ios")]

use objc2::encode::{Encode, Encoding, RefEncode};
use objc2::msg_send;
use objc2::runtime::AnyObject;
use std::time::Duration;
use tauri::Manager;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

// On 64-bit Apple platforms `CGFloat` is `double`, so the Objective-C type
// encoding for these structs is just two/four doubles in the documented order.
// We need these manual `Encode` impls because `objc2::msg_send!` checks the
// argument/return ABI at compile time and won't accept arbitrary `#[repr(C)]`
// types without them.
unsafe impl Encode for CGPoint {
    const ENCODING: Encoding =
        Encoding::Struct("CGPoint", &[<f64 as Encode>::ENCODING, <f64 as Encode>::ENCODING]);
}
unsafe impl RefEncode for CGPoint {
    const ENCODING_REF: Encoding = Encoding::Pointer(&<Self as Encode>::ENCODING);
}
unsafe impl Encode for CGSize {
    const ENCODING: Encoding =
        Encoding::Struct("CGSize", &[<f64 as Encode>::ENCODING, <f64 as Encode>::ENCODING]);
}
unsafe impl RefEncode for CGSize {
    const ENCODING_REF: Encoding = Encoding::Pointer(&<Self as Encode>::ENCODING);
}
unsafe impl Encode for CGRect {
    const ENCODING: Encoding = Encoding::Struct(
        "CGRect",
        &[<CGPoint as Encode>::ENCODING, <CGSize as Encode>::ENCODING],
    );
}
unsafe impl RefEncode for CGRect {
    const ENCODING_REF: Encoding = Encoding::Pointer(&<Self as Encode>::ENCODING);
}

/// `UIScrollViewContentInsetAdjustmentNever` raw value.
const CONTENT_INSET_ADJUSTMENT_NEVER: isize = 2;

/// `UIViewAutoresizing.flexibleWidth | .flexibleHeight`.
const AUTORESIZE_FILL: usize = (1 << 1) | (1 << 4);

/// Applies the layout fix to a raw `WKWebView*`. Safe to call multiple times.
///
/// `webview_ptr` is the value handed in by `tauri::webview::PlatformWebview::inner()`,
/// which on iOS is a non-owning Objective-C pointer to `WKWebView`.
unsafe fn apply_to_webview(webview_ptr: *mut std::ffi::c_void) {
    if webview_ptr.is_null() {
        return;
    }
    let webview = webview_ptr as *mut AnyObject;

    // Step 1: never auto-adjust scrollview content insets (this is what causes
    // wry/iOS to push the page down by `safeAreaInsets.top` and pad the bottom).
    let scroll_view: *mut AnyObject = msg_send![webview, scrollView];
    if !scroll_view.is_null() {
        let _: () = msg_send![
            scroll_view,
            setContentInsetAdjustmentBehavior: CONTENT_INSET_ADJUSTMENT_NEVER
        ];
    }

    // Step 2: match the parent view's full bounds (which IS the full screen);
    // wry pins us to safeAreaLayoutGuide.layoutFrame by default.
    let superview: *mut AnyObject = msg_send![webview, superview];
    if superview.is_null() {
        return;
    }

    let bounds: CGRect = msg_send![superview, bounds];
    let _: () = msg_send![webview, setTranslatesAutoresizingMaskIntoConstraints: true];
    let _: () = msg_send![webview, setFrame: bounds];
    let _: () = msg_send![webview, setAutoresizingMask: AUTORESIZE_FILL];
    let _: () = msg_send![webview, setNeedsLayout];
}

/// Public entry point. Schedules the fix to be applied right now, then again
/// at +500ms and +1500ms in case iOS relayouts after we return.
///
/// Pass in the value of `tauri::webview::PlatformWebview::inner()`. The pointer
/// is treated as borrowed — we never release it.
pub fn install(app: &tauri::AppHandle) {
    let Some(main) = app.get_webview_window("main") else {
        return;
    };

    let snapshot: std::sync::Arc<std::sync::atomic::AtomicUsize> =
        std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    // `with_webview` runs synchronously and the pointer is only valid for the
    // duration of the closure. Capture it as a usize so we can re-fetch on
    // subsequent ticks via another `with_webview` call.
    let _ = main.with_webview({
        let snapshot = snapshot.clone();
        move |w| {
            let ptr = w.inner() as usize;
            snapshot.store(ptr, std::sync::atomic::Ordering::Release);
            unsafe { apply_to_webview(ptr as *mut std::ffi::c_void) };
        }
    });

    // Re-apply asynchronously. We *re-enter* `with_webview` (which dispatches
    // onto the main UI thread internally) so all Objective-C work happens on
    // the right queue.
    let app = app.clone();
    std::thread::spawn(move || {
        for delay in [Duration::from_millis(500), Duration::from_millis(1500)] {
            std::thread::sleep(delay);
            let Some(win) = app.get_webview_window("main") else {
                return;
            };
            let _ = win.with_webview(|w| unsafe {
                apply_to_webview(w.inner() as *mut std::ffi::c_void);
            });
        }
    });
}
