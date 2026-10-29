//! User-initiated GDI capture with in-memory Windows Imaging Component encoding.
//! No shell, dynamic compilation, temporary image or security-policy changes.
use super::{encode_base64, Frame};
use std::{
    ffi::c_void,
    mem::size_of,
    ptr::{null, null_mut},
};
use windows::Win32::{
    Foundation::{HGLOBAL, RPC_E_CHANGED_MODE},
    Graphics::Imaging::{
        CLSID_WICImagingFactory, GUID_ContainerFormatJpeg, GUID_WICPixelFormat24bppBGR,
        IWICImagingFactory, WICBitmapEncoderNoCache,
    },
    System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize,
        StructuredStorage::{CreateStreamOnHGlobal, IPropertyBag2},
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, STATFLAG_NONAME, STATSTG, STREAM_SEEK_SET,
    },
};
use windows_sys::Win32::{
    Foundation::{LPARAM, RECT},
    Graphics::Gdi::*,
    UI::HiDpi::{
        SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT,
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    },
};

const MAX_RAW_BYTES: usize = 128 * 1024 * 1024;
const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 32 * 1024 * 1024;

fn failure(operation: &str) -> String {
    format!(
        "Windows 截图失败（{operation}）：{}。请确认当前桌面已解锁且允许截图。",
        std::io::Error::last_os_error()
    )
}
fn codec_error(error: windows::core::Error) -> String {
    format!("截图图像处理失败：{error}")
}

struct DpiScope(DPI_AWARENESS_CONTEXT);
impl DpiScope {
    fn enter() -> Result<Self, String> {
        let mut old =
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        if old.is_null() {
            old = unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE) };
        }
        if old.is_null() {
            Err(failure("设置截图坐标"))
        } else {
            Ok(Self(old))
        }
    }
}
impl Drop for DpiScope {
    fn drop(&mut self) {
        unsafe {
            SetThreadDpiAwarenessContext(self.0);
        }
    }
}

struct ComScope(bool);
impl ComScope {
    fn enter() -> Result<Self, String> {
        let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if result.is_ok() {
            Ok(Self(true))
        } else if result == RPC_E_CHANGED_MODE {
            Ok(Self(false))
        } else {
            Err(codec_error(windows::core::Error::from_hresult(result)))
        }
    }
}
impl Drop for ComScope {
    fn drop(&mut self) {
        if self.0 {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

struct ScreenDc(HDC);
impl Drop for ScreenDc {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(null_mut(), self.0);
        }
    }
}
struct Bitmap {
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
}
impl Drop for Bitmap {
    fn drop(&mut self) {
        unsafe {
            if !self.previous.is_null() && self.previous as isize != -1 {
                SelectObject(self.dc, self.previous);
            }
            if !self.bitmap.is_null() {
                DeleteObject(self.bitmap);
            }
            if !self.dc.is_null() {
                DeleteDC(self.dc);
            }
        }
    }
}

#[derive(Default)]
struct Monitors {
    rectangles: Vec<RECT>,
    overflow: bool,
}
unsafe extern "system" fn collect_monitor(
    _: HMONITOR,
    _: HDC,
    rect: *mut RECT,
    data: LPARAM,
) -> i32 {
    let monitors = unsafe { &mut *(data as *mut Monitors) };
    if rect.is_null() {
        return 0;
    }
    if monitors.rectangles.len() >= 16 {
        monitors.overflow = true;
        return 0;
    }
    monitors.rectangles.push(unsafe { *rect });
    1
}

fn layout(width: i32, height: i32) -> Result<(u32, usize), String> {
    if width <= 0 || height <= 0 {
        return Err("显示器尺寸无效，请重新连接显示器后重试。".into());
    }
    let stride = (width as usize)
        .checked_mul(3)
        .and_then(|n| n.checked_add(3))
        .map(|n| n & !3)
        .ok_or("屏幕画面过大")?;
    let bytes = stride.checked_mul(height as usize).ok_or("屏幕画面过大")?;
    if bytes > MAX_RAW_BYTES || stride > u32::MAX as usize {
        return Err("屏幕画面过大，请降低分辨率后重试。".into());
    }
    Ok((stride as u32, bytes))
}

fn jpeg(width: i32, height: i32, stride: u32, pixels: &[u8]) -> Result<Vec<u8>, String> {
    let (expected_stride, size) = layout(width, height)?;
    if stride != expected_stride || pixels.len() != size {
        return Err("截图像素数据不完整。".into());
    }
    let _com = ComScope::enter()?;
    unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
                .map_err(codec_error)?;
        let stream = CreateStreamOnHGlobal(HGLOBAL::default(), true).map_err(codec_error)?;
        let encoder = factory
            .CreateEncoder(&GUID_ContainerFormatJpeg, null())
            .map_err(codec_error)?;
        encoder
            .Initialize(&stream, WICBitmapEncoderNoCache)
            .map_err(codec_error)?;
        let mut frame = None;
        encoder
            .CreateNewFrame(&mut frame, null_mut())
            .map_err(codec_error)?;
        let frame = frame.ok_or("无法创建截图编码器")?;
        frame
            .Initialize(None::<&IPropertyBag2>)
            .map_err(codec_error)?;
        frame
            .SetSize(width as u32, height as u32)
            .map_err(codec_error)?;
        let mut format = GUID_WICPixelFormat24bppBGR;
        frame.SetPixelFormat(&mut format).map_err(codec_error)?;
        if format != GUID_WICPixelFormat24bppBGR {
            return Err("系统截图编码器不支持当前像素格式。".into());
        }
        frame
            .WritePixels(height as u32, stride, pixels)
            .map_err(codec_error)?;
        frame.Commit().map_err(codec_error)?;
        encoder.Commit().map_err(codec_error)?;
        let mut stat = STATSTG::default();
        stream
            .Stat(&mut stat, STATFLAG_NONAME)
            .map_err(codec_error)?;
        let length = usize::try_from(stat.cbSize).map_err(|_| "截图图像过大")?;
        if length == 0 || length > MAX_FRAME_BYTES {
            return Err("截图图像过大，请降低分辨率后重试。".into());
        }
        stream.Seek(0, STREAM_SEEK_SET, None).map_err(codec_error)?;
        let mut bytes = vec![0; length];
        let mut read = 0;
        stream
            .Read(bytes.as_mut_ptr().cast(), length as u32, Some(&mut read))
            .ok()
            .map_err(codec_error)?;
        if read as usize != length || !bytes.starts_with(&[0xff, 0xd8]) {
            return Err("截图图像编码不完整。".into());
        }
        Ok(bytes)
    }
}

fn capture_monitor(screen: &ScreenDc, rect: RECT) -> Result<Vec<u8>, String> {
    let width = rect.right.checked_sub(rect.left).ok_or("显示器坐标无效")?;
    let height = rect.bottom.checked_sub(rect.top).ok_or("显示器坐标无效")?;
    let (stride, size) = layout(width, height)?;
    unsafe {
        let mut bitmap = Bitmap {
            dc: CreateCompatibleDC(screen.0),
            bitmap: null_mut(),
            previous: null_mut(),
        };
        if bitmap.dc.is_null() {
            return Err(failure("创建截图缓冲区"));
        }
        let mut info = BITMAPINFO::default();
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 24,
            biCompression: BI_RGB,
            ..Default::default()
        };
        let mut pixels: *mut c_void = null_mut();
        bitmap.bitmap =
            CreateDIBSection(screen.0, &info, DIB_RGB_COLORS, &mut pixels, null_mut(), 0);
        if bitmap.bitmap.is_null() || pixels.is_null() {
            return Err(failure("分配截图缓冲区"));
        }
        bitmap.previous = SelectObject(bitmap.dc, bitmap.bitmap);
        if bitmap.previous.is_null() || bitmap.previous as isize == -1 {
            return Err(failure("准备屏幕读取"));
        }
        if BitBlt(
            bitmap.dc,
            0,
            0,
            width,
            height,
            screen.0,
            rect.left,
            rect.top,
            SRCCOPY | CAPTUREBLT,
        ) == 0
        {
            return Err(failure("读取屏幕画面"));
        }
        if GdiFlush() == 0 {
            return Err(failure("完成屏幕读取"));
        }
        // GDI has completed writing, and Bitmap owns this top-down buffer until encoding returns.
        let pixels = std::slice::from_raw_parts(pixels.cast::<u8>(), size);
        jpeg(width, height, stride, pixels)
    }
}

pub(super) fn capture() -> Result<Vec<Frame>, String> {
    let _dpi = DpiScope::enter()?;
    let mut monitors = Monitors::default();
    let enumerated = unsafe {
        EnumDisplayMonitors(
            null_mut(),
            null(),
            Some(collect_monitor),
            &mut monitors as *mut Monitors as LPARAM,
        )
    };
    if monitors.overflow {
        return Err("显示器数量超过截图支持范围".into());
    }
    if enumerated == 0 {
        return Err(failure("读取显示器列表"));
    }
    if monitors.rectangles.is_empty() {
        return Err("没有可截图的显示器，请回到已登录的桌面后重试。".into());
    }
    let screen = ScreenDc(unsafe { GetDC(null_mut()) });
    if screen.0.is_null() {
        return Err(failure("连接当前桌面"));
    }
    let mut total = 0;
    let mut frames = Vec::new();
    for rect in monitors.rectangles {
        let bytes = capture_monitor(&screen, rect)?;
        total += bytes.len();
        if total > MAX_TOTAL_BYTES {
            return Err("屏幕画面过大，请降低分辨率后重试。".into());
        }
        frames.push(Frame {
            x: rect.left as f64,
            y: rect.top as f64,
            width: (rect.right - rect.left) as f64,
            height: (rect.bottom - rect.top) as f64,
            image: format!("data:image/jpeg;base64,{}", encode_base64(&bytes)),
        });
    }
    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_stride_supports_odd_widths_and_rejects_huge_frames() {
        assert_eq!(layout(1, 2).unwrap(), (4, 8));
        assert_eq!(layout(1920, 1080).unwrap(), (5760, 6_220_800));
        for (w, h) in [(0, 1), (-1, 1), (i32::MAX, i32::MAX)] {
            assert!(layout(w, h).is_err());
        }
    }
    #[test]
    fn native_jpeg_encoding_accepts_padded_bgr_and_rejects_partial_pixels() {
        let pixels = [0, 0, 255, 0, 255, 0, 0, 0, 255, 0, 0, 255, 255, 255, 0, 0];
        let encoded = jpeg(2, 2, 8, &pixels).unwrap();
        assert!(encoded.starts_with(&[0xff, 0xd8]) && encoded.ends_with(&[0xff, 0xd9]));
        assert!(jpeg(2, 2, 8, &pixels[..8]).is_err());
    }
    #[test]
    #[ignore = "captures the current interactive desktop; run explicitly for native validation"]
    fn native_capture_current_desktop() {
        let frames = capture().unwrap();
        assert!(!frames.is_empty());
        for frame in frames {
            assert!(
                frame.width > 0.0
                    && frame.height > 0.0
                    && frame.image.starts_with("data:image/jpeg;base64,/9j/")
            );
        }
    }
}
