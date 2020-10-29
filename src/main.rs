mod compression;
mod game;

use game::Game;

use image::Luma;
use num_bigint::BigUint;
use qrcode::QrCode;

fn main() {
    let game = Game::create_state(None);

    let serialized = ron::to_string(&game).unwrap();
    let serialized_bytes = bincode::serialize(&game).unwrap();

    // Encode some data into bits.
    let code = QrCode::new(&vec![0]).unwrap();

    // Render the bits into an image.
    let image = code.render::<Luma<u8>>().build();

    // Save the image.
    image.save("./qrcode.png").unwrap();

    let image = image::open("./qrcode.png").expect("failed to open image");

    // convert to gray scale
    let img_gray = image.into_luma();

    // create a decoder
    let mut decoder = quircs::Quirc::default();

    // identify all qr codes
    let codes = decoder.identify(
        img_gray.width() as usize,
        img_gray.height() as usize,
        &img_gray,
    );

    for code in codes {
        let code = code.expect("failed to extract qr code");
        let decoded = code.decode().expect("failed to decode qr code");
        println!("qrcode: {:?}", &decoded.payload);
    }
}
