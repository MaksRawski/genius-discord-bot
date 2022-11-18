use magick_rust::bindings::{
    CompositeOperator_BlendCompositeOp, CompositeOperator_SrcOverCompositeOp,
    FilterType_LanczosFilter, GravityType_CenterGravity, GravityType_SouthWestGravity,
    StyleType_NormalStyle,
};
use magick_rust::{DrawingWand, MagickWand, PixelWand};
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
) -> Result<String, anyhow::Error> {
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

    // it's safe to unwrap (take first element of the result) since
    // we know that the length will be over 34 so the result can't be empty
    if title_up.len() > 34 {
        title_up = textwrap::wrap(&title_up, 34)[0].to_string().add("...");
    }
    if artist_up.len() > 34 {
        artist_up = textwrap::wrap(&artist_up, 34)[0].to_string().add("...");
    }
    let card_info = format!("{} \"{}\"", artist_up, title_up);

    let mut bars = textwrap::wrap(quote, 30);
    let mut bar_font_size = 52;
    let mut bar_gap = 90;
    let mut last_bar_y = 405;
    let bar_w_pad = 5;

    if bars.len() > 4 {
        bars = textwrap::wrap(quote, 46);
        bar_font_size = 35;
        bar_gap = 62;

        last_bar_y = 425;
    }

    if card_info.len() > 40 {
        d_wand.draw_annotation(90.0, 70.0, &artist_up)?;
        d_wand.draw_annotation(90.0, 32.0, &format!("\"{}\"", &title_up))?;
        // last_bar_y -= 10;
    } else {
        d_wand.draw_annotation(90.0, 52.0, &card_info)?;
    }

    // 4. create bars
    for (i, bar) in bars.iter().enumerate() {
        let mut bar_wand = MagickWand::new();

        bar_wand.set_font("Lato-SemiBold")?;
        bar_wand.set_pointsize(bar_font_size as f64)?;
        bar_wand.read_image(&format!("label:{}", bar))?;
        // when running locally (at least on my machine) it looks fine with just border of 1px in height
        // however when running through docker, there seems to be pretty much no border and 7 seems do the trick
        bar_wand.border_image(&p_wand, bar_w_pad, 7, CompositeOperator_SrcOverCompositeOp)?;

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
        30,
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
    info!("It took {:.2?} to create the card.", start.elapsed());
    Ok(filename)
}
