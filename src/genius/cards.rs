use magick_rust::bindings::{
    CompositeOperator_BlendCompositeOp, CompositeOperator_SrcOverCompositeOp,
    FilterType_LanczosFilter, GravityType_CenterGravity, GravityType_SouthWestGravity,
    MagickBooleanType_MagickTrue, StyleType_NormalStyle,
};
use magick_rust::{DrawingWand, MagickError, MagickWand, PixelWand};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::ops::Add;
use std::time::Instant;
use tracing::info;

const QUOTE_SYMBOL: &[u8] = include_bytes!("../../scripts/quote.png");

/// returns either path to a generated card
/// or a MagickError describing why it failed
/// expects quote to be shorter than 400 chars
pub fn generate_card(
    img_path: &str,
    quote: &str,
    artist: &str,
    title: &str,
) -> Result<String, MagickError> {
    let start = Instant::now();

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
        wand.resize_image(900, 900 / (width / height), FilterType_LanczosFilter);
        wand.extend_image(900, 600, 0, 150)?;
    } else {
        wand.resize_image(width / height * 600, 600, FilterType_LanczosFilter);
        wand.extend_image(900, 600, 225, 0)?;
    }
    // make sure it's cropped
    wand.crop_image(900, 600, 0, 0)?;

    // 2. darken the image
    wand.brightness_contrast_image(-20.0, -25.0)?;

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

    let mut title_up = title.to_uppercase();
    let mut artist_up = artist.to_uppercase();

    if title_up.len() > 34 {
        title_up.truncate(34);
        title_up = title_up.add("...");
    }
    if artist_up.len() > 34 {
        artist_up.truncate(34);
        artist_up = artist_up.add("...");
    }
    let card_info = format!("{} \"{}\"", artist_up, title_up);

    if card_info.len() > 30 {
        d_wand.draw_annotation(90.0, 75.0, &artist_up)?;
        d_wand.draw_annotation(90.0, 40.0, &format!("\"{}\"", &title_up))?;
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
        bar_d_wand.set_font_weight(400);
        bar_d_wand.set_font_style(StyleType_NormalStyle);
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
    quote_symbol.read_image_blob(QUOTE_SYMBOL)?;

    let top_bar = last_bar_y - (bars.len() - 1) as isize * bar_gap as isize;
    wand.compose_images(
        &quote_symbol,
        CompositeOperator_BlendCompositeOp,
        true,
        25,
        top_bar,
    )?;

    wand.draw_image(&d_wand)?;

    let filename: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect::<String>()
        .add(".jpg");

    wand.write_image(&filename)?;
    info!("It took {:.2?} to create a card.", start.elapsed());
    Ok(filename)
}
