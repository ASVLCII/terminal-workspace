#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod fonts;
mod foundation;
mod icons;
fn main() -> eframe::Result {
    if std::env::args().any(|a| a == "--foundation" || a == "--opencode-gate") {
        return foundation::run();
    }
    let Some(instance) = terminal_workspace::instance::Instance::acquire()
        .map_err(|e| eframe::Error::AppCreation(e.into()))?
    else {
        return Ok(());
    };
    let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
    if std::env::var_os("WGPU_BACKEND").is_none() {
        setup.instance_descriptor.backends = eframe::wgpu::Backends::DX12;
    }
    setup.power_preference = eframe::wgpu::PowerPreference::LowPower;
    let descriptor = setup.device_descriptor.clone();
    setup.device_descriptor = std::sync::Arc::new(move |adapter| {
        let mut device = descriptor(adapter);
        device.memory_hints = eframe::wgpu::MemoryHints::MemoryUsage;
        device
    });
    let options = eframe::NativeOptions {
        wgpu_options: eframe::WgpuConfiguration {
            wgpu_setup: setup.into(),
            ..Default::default()
        },
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Terminal Workspace")
            .with_inner_size(if std::env::args().any(|a| a == "--benchmark-workload") {
                [1600., 950.]
            } else {
                [1280., 800.]
            })
            .with_min_inner_size([800., 520.]),
        ..Default::default()
    };
    eframe::run_native(
        "Terminal Workspace",
        options,
        Box::new(move |cc| {
            instance.listen(cc.egui_ctx.clone());
            Ok(Box::new(app::App::new(cc, instance)?))
        }),
    )
}
