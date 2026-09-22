use fontdue::*;
fn main() {
    let bytes = std::fs::read("assets/fonts/OpenSans-Regular.ttf").unwrap();
    let font = Font::from_bytes(bytes, FontSettings::default()).unwrap();
    let tofu = '\u{25A1}';
    println!("TOFU index: {}", font.lookup_glyph_index(tofu));
    if font.lookup_glyph_index(tofu) == 0 {
        println!("OpenSans does NOT have TOFU");
    } else {
        println!("OpenSans HAS TOFU");
    }
}
