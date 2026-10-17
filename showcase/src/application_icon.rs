// SPDX-License-Identifier: GPL-3.0-or-later

/// Give an unbundled `cargo run` process its own macOS Dock icon.
/// Call on the main thread after Slint has initialized the native application.
pub fn install() -> Result<(), slint::PlatformError> {
    #[cfg(target_os = "macos")]
    {
        use objc2::{AnyThread, MainThreadMarker};
        use objc2_app_kit::{NSApplication, NSImage};
        use objc2_foundation::NSData;

        let main_thread = MainThreadMarker::new()
            .ok_or("The showcase application icon must be installed on the main thread")?;
        let pixels = slint::Image::load_from_svg_data(include_bytes!(
            "../../assets/branding/slint-kit-icon.svg"
        ))
        .map_err(|error| format!("Cannot load the showcase application icon: {error}"))?
        .to_rgba8()
        .ok_or("Cannot rasterize the showcase application icon")?;

        let mut encoded = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut encoded, pixels.width(), pixels.height());
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder
                .write_header()
                .map_err(|error| format!("Cannot encode the showcase application icon: {error}"))?;
            writer
                .write_image_data(pixels.as_bytes())
                .map_err(|error| format!("Cannot encode the showcase application icon: {error}"))?;
        }

        objc2::rc::autoreleasepool(|_| {
            let data = NSData::with_bytes(&encoded);
            let image = NSImage::initWithData(NSImage::alloc(), &data)
                .ok_or("AppKit cannot decode the showcase application icon")?;
            let application = NSApplication::sharedApplication(main_thread);
            // SAFETY: AppKit is accessed on the main thread and the image is non-null.
            // NSApplication retains the image after this local reference is released.
            unsafe { application.setApplicationIconImage(Some(&image)) };
            Ok::<(), slint::PlatformError>(())
        })?;
    }
    Ok(())
}
