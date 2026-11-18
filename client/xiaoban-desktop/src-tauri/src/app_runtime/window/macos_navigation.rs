use block2::DynBlock;
use objc2::{
    define_class, ffi, msg_send,
    rc::{Retained, Weak},
    runtime::{AnyObject, NSObject, NSObjectProtocol, ProtocolObject, Sel},
    sel, DeclaredClass, MainThreadMarker, MainThreadOnly,
};
use objc2_foundation::{NSString, NSURLRequest, NSURL};
use objc2_web_kit::{
    WKNavigationAction, WKNavigationActionPolicy, WKNavigationDelegate, WKWebView,
};
use tauri::WebviewWindow;

static DELEGATE_KEY: u8 = 0;

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = Weak<ProtocolObject<dyn WKNavigationDelegate>>]
    struct AppRuntimeNavigationDelegate;

    unsafe impl NSObjectProtocol for AppRuntimeNavigationDelegate {}

    unsafe impl WKNavigationDelegate for AppRuntimeNavigationDelegate {
        #[unsafe(method(webView:decidePolicyForNavigationAction:decisionHandler:))]
        fn decide_navigation(
            &self,
            webview: &WKWebView,
            action: &WKNavigationAction,
            decision: &DynBlock<dyn Fn(WKNavigationActionPolicy)>,
        ) {
            // Wry's URL-only callback handles both types of navigation on macOS. Bypass it
            // only for HTTPS subframes, and keep Wry's policy for the package's main frame.
            let is_https_subframe = unsafe {
                action.targetFrame().is_some_and(|frame| !frame.isMainFrame())
                    && action.request().URL().and_then(|url| url.scheme())
                        .is_some_and(|scheme| scheme.to_string() == "https")
                    && !(action.respondsToSelector(sel!(shouldPerformDownload))
                        && action.shouldPerformDownload())
            };
            if is_https_subframe {
                decision.call((WKNavigationActionPolicy::Allow,));
            } else if let Some(original) = self.ivars().load() {
                unsafe {
                    original.webView_decidePolicyForNavigationAction_decisionHandler(webview, action, decision);
                }
            } else {
                decision.call((WKNavigationActionPolicy::Cancel,));
            }
        }
    }

    impl AppRuntimeNavigationDelegate {
        #[unsafe(method(respondsToSelector:))]
        fn responds(&self, selector: Sel) -> bool {
            let own: bool = unsafe { msg_send![super(self), respondsToSelector: selector] };
            own || self.ivars().load().is_some_and(|original| original.respondsToSelector(selector))
        }

        #[unsafe(method(forwardingTargetForSelector:))]
        fn forwarding_target(&self, selector: Sel) -> *mut AnyObject {
            self.ivars().load()
                .filter(|original| original.respondsToSelector(selector))
                .map_or(std::ptr::null_mut(), |original| Retained::autorelease_ptr(original).cast())
        }
    }
);

pub(super) fn open(window: &WebviewWindow, entry_url: url::Url) -> Result<(), String> {
    let window_on_error = window.clone();
    window
        .with_webview(move |view| {
            // Tauri provides the native WKWebView on its main thread for this callback.
            let webview = unsafe { &*view.inner().cast::<WKWebView>() };
            if let Err(error) = attach_and_load(webview, &entry_url) {
                eprintln!("应用窗口导航初始化失败: {error}");
                let _ = window_on_error.close();
            }
        })
        .map_err(|error| format!("应用窗口导航初始化失败: {error}"))
}

fn attach_and_load(webview: &WKWebView, entry: &url::Url) -> Result<(), &'static str> {
    let mtm = MainThreadMarker::from(webview);
    let url = NSURL::URLWithString(&NSString::from_str(entry.as_str())).ok_or("作品入口无效")?;
    let original = unsafe { webview.navigationDelegate() }.ok_or("作品窗口缺少导航代理")?;
    let delegate = mtm
        .alloc::<AppRuntimeNavigationDelegate>()
        .set_ivars(Weak::new(&*original));
    let delegate: Retained<AppRuntimeNavigationDelegate> =
        unsafe { msg_send![super(delegate), init] };

    // WKWebView holds its delegate weakly. Attach ours to its lifetime while keeping
    // Wry's delegate weak here to avoid retaining the view through Wry's callbacks.
    unsafe {
        ffi::objc_setAssociatedObject(
            std::ptr::from_ref(webview).cast_mut().cast(),
            std::ptr::from_ref(&DELEGATE_KEY).cast(),
            Retained::as_ptr(&delegate).cast_mut().cast(),
            ffi::OBJC_ASSOCIATION_RETAIN_NONATOMIC,
        );
        webview.setNavigationDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        webview
            .loadRequest(&NSURLRequest::requestWithURL(&url))
            .ok_or("无法加载作品入口")?;
    }
    Ok(())
}
