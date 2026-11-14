pub fn enable(cc: &eframe::CreationContext<'_>) -> bool {
    #[cfg(windows)]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use windows_sys::Win32::Graphics::Dwm::{
            DWMSBT_NONE, DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE,
            DwmSetWindowAttribute,
        };

        let composition = if std::env::var_os("WGPU_BACKEND").is_none() {
            true
        } else {
            eframe::wgpu::Backends::from_env() == Some(eframe::wgpu::Backends::DX12)
        };
        if !composition
            || !cc.wgpu_render_state.as_ref().is_some_and(|state| {
                state.adapter.get_info().backend == eframe::wgpu::Backend::Dx12
            })
        {
            return false;
        }
        let Ok(handle) = cc.window_handle() else {
            return false;
        };
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return false;
        };
        let hwnd = handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
        let dark = 1i32;
        // The premultiplied DX12 swapchain supplies the glass tint. Disable
        // the system material so Acrylic cannot cover the desktop with a
        // second tint or replace it with a solid fallback.
        let backdrop = DWMSBT_NONE;
        // The HWND is borrowed from the live eframe window. DWM copies both values.
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                (&raw const dark).cast(),
                4,
            );
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE as u32,
                (&raw const backdrop).cast(),
                4,
            );
        }
        true
    }
    #[cfg(not(windows))]
    {
        let _ = cc;
        false
    }
}
