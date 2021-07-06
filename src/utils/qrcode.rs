use qrcode::render::unicode::Dense1x2;
use qrcode::QrCode;

pub(crate) fn qrcode_image<D: AsRef<[u8]>>(data: D, inverted: bool) -> String {
    let code = QrCode::new(data).unwrap();
    let mut rederer = code.render::<Dense1x2>();

    if inverted {
        rederer
            .dark_color(Dense1x2::Light)
            .light_color(Dense1x2::Dark);
    }

    rederer.build()
}
