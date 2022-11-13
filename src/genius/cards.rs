use magick_rust::bindings::{
    CompositeOperator_BlendCompositeOp, CompositeOperator_SrcOverCompositeOp,
    FilterType_LanczosFilter, GravityType_CenterGravity, GravityType_SouthWestGravity,
    MagickBooleanType_MagickTrue, StyleType_NormalStyle,
};
use magick_rust::{DrawingWand, MagickError, MagickWand, PixelWand};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::ops::Add;
use std::process::Command;
use tracing::error;

/// returns filename of the output image
/// img must exist and be valid, no checks are done!
pub fn old_generate_card(img: &str, caption: &str, author: &str, track: &str) -> Option<String> {
    let filename: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect::<String>()
        .add(".jpg");

    Command::new("./scripts/generate.sh")
        .args([img, caption, author, track, &filename])
        .status()
        .map_err(|e| error!("Error in card generation: {}", e))
        .ok()?;

    Some(filename)
}

// TODO rewrite in https://crates.io/crates/andrew
/// returns either path to a generated card
/// or a MagickError describing why it failed
/// expects quote to be shorter than 400 chars
pub fn generate_card(
    img_path: &str,
    quote: &str,
    artist: &str,
    title: &str,
) -> Result<String, MagickError> {
    // 0. load the image
    let mut wand = MagickWand::new();
    let mut p_wand = PixelWand::new();
    wand.read_image(img_path)?;

    // 1. resize
    let width = wand.get_image_width();
    let height = wand.get_image_height();

    p_wand.set_color("white")?;
    wand.set_background_color(&p_wand)?;
    wand.set_gravity(GravityType_CenterGravity)?;

    if width >= height {
        wand.resize_image(900, height / width * 900, FilterType_LanczosFilter);
        wand.extend_image(900, 600, 0, 160)?;
    } else {
        wand.resize_image(width / height * 600, 600, FilterType_LanczosFilter);
        wand.extend_image(900, 600, 225, 0)?;
    }
    // make sure it's cropped
    wand.crop_image(900, 600, 0, 0)?;

    // 2. darken the image
    wand.brightness_contrast_image(-15.0, -15.0)?;

    // 3. add author and title
    let mut d_wand = DrawingWand::new();
    d_wand.set_font("Lato")?;
    d_wand.set_font_family("Lato")?;
    d_wand.set_font_style(StyleType_NormalStyle);
    d_wand.set_font_weight(450);
    d_wand.set_font_size(32.0);
    d_wand.set_text_kerning(2.0);

    // d_wand.set_text_antialias(MagickBooleanType_MagickTrue);
    d_wand.set_fill_color(&p_wand);
    d_wand.set_stroke_color(&p_wand);

    wand.set_gravity(GravityType_SouthWestGravity)?;

    let card_info = format!("{} \"{}\"", artist.to_uppercase(), title.to_uppercase());

    if card_info.len() > 30 {
        d_wand.draw_annotation(90.0, 75.0, &artist.to_uppercase())?;
        // TODO add dots if the title itself is above 30 chars
        d_wand.draw_annotation(90.0, 40.0, &format!("\"{}\"", &title.to_uppercase()))?;
    } else {
        d_wand.draw_annotation(90.0, 52.0, &card_info)?;
    }

    let mut bars = textwrap::wrap(quote, 30);
    let mut bar_height = 68;
    let mut bar_gap = bar_height + 22;
    let mut bar_font_size = 50;
    let mut last_bar_y = 405;
    let mut bar_w_pad = 10;

    // // for quotes longer than 120 chars, wrap at 50
    if quote.len() > 120 {
        bars = textwrap::wrap(quote, 48);
        bar_height = 46;
        bar_gap = bar_height + 12;
        bar_font_size = 32;

        bar_w_pad = 5;
        last_bar_y = 425;
    }

    // 4. create bars
    let mut bar_p_wand = PixelWand::new();
    // each iteration takes about 30ms
    // can something be done about it? i don't know :/
    for (i, bar) in bars.iter().enumerate() {
        let mut bar_wand = MagickWand::new();
        let mut bar_d_wand = DrawingWand::new();

        bar_p_wand.set_color("white")?;
        bar_wand.new_image(800, 200, &bar_p_wand).unwrap();

        bar_p_wand.set_color("black")?;
        bar_d_wand.set_font("Lato")?;
        bar_d_wand.set_font_family("Lato")?;
        bar_d_wand.set_font_weight(500);
        bar_d_wand.set_font_size(bar_font_size as f64);
        bar_d_wand.set_fill_color(&bar_p_wand);
        bar_d_wand.set_stroke_color(&bar_p_wand);
        bar_d_wand.set_text_kerning(2.0);
        bar_d_wand.set_text_antialias(MagickBooleanType_MagickTrue);

        bar_d_wand.draw_annotation(5.0, 100.0, &bar)?;
        bar_wand.draw_image(&bar_d_wand)?;

        bar_wand.trim_image(0.0)?;
        bar_wand.extend_image(
            bar_wand.get_image_width() + bar_w_pad * 2,
            bar_height as usize,
            -(bar_w_pad as isize),
            -bar_height / 4,
        )?;

        wand.compose_images(
            &bar_wand,
            CompositeOperator_SrcOverCompositeOp,
            true,
            90,
            last_bar_y - (bars.len() - i - 1) as isize * bar_gap as isize,
        )?;
    }

    // 5. add quote symbol to first bar
    let quote_symbol = MagickWand::new();
    quote_symbol.read_image("scripts/quote.png")?;
    quote_symbol.resize_image(42, 32, FilterType_LanczosFilter);

    let top_bar = last_bar_y - (bars.len() - 1) as isize * bar_gap as isize;
    wand.compose_images(
        &quote_symbol,
        CompositeOperator_BlendCompositeOp,
        true,
        25,
        top_bar,
    )?;

    wand.draw_image(&d_wand)?;
    wand.write_image(&format!("{}.png", artist))?;
    Ok("asdf.png".to_owned())
}
