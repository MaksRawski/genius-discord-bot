use ab_glyph::{Font, FontArc, ScaleFont};
use image::{imageops, DynamicImage};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut, text_size},
    rect::Rect,
};
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;

const LATO_SEMIBOLD: &[u8] = include_bytes!("../../resources/Lato/Lato-Semibold.ttf");
const LATO_BOLD: &[u8] = include_bytes!("../../resources/Lato/Lato-Bold.ttf");
const QUOTE_SYMBOL: &[u8] = include_bytes!("../../resources/quote.png");

pub fn generate_card(
    img_data: DynamicImage,
    caption: &str,
    artist: &str,
    title: &str,
) -> Result<String, anyhow::Error> {
    // 0. Load resources
    let lato_semibold = FontArc::try_from_slice(LATO_SEMIBOLD)?;
    let lato_bold = FontArc::try_from_slice(LATO_BOLD)?;
    let mut img = img_data.into_rgb8();

    // 1. resize
    let (w, h) = img.dimensions();
    img = imageops::resize(&img, 900, h * 900 / w, imageops::Lanczos3);

    let cropped = DynamicImage::ImageRgb8(
        imageops::crop_imm(
            &img,
            (img.width() - 900) / 2,
            (img.height() - 600) / 2,
            900,
            600,
        )
        .to_image(),
    );

    // 2. darken image
    let dark = &cropped.adjust_contrast(-20.0).brighten(-40).into_rgb8();
    let mut canvas: DynamicImage = DynamicImage::ImageRgb8(dark.clone());

    // 3. artist and title
    let mut title_up = title.to_uppercase();
    let mut artist_up = artist.to_uppercase();
    if title_up.len() > 34 {
        title_up = textwrap::wrap(&title_up, 34)[0].to_string() + "...";
    }
    if artist_up.len() > 34 {
        artist_up = textwrap::wrap(&artist_up, 34)[0].to_string() + "...";
    }
    let card_info = format!("{} \"{}\"", artist_up, title_up);
    let info_font_size = 41.0;

    if card_info.len() > 40 {
        // Draw artist and title separately
        let y_artist = 600.0 - 101.0;
        draw_text_mut(
            &mut canvas,
            image::Rgba([255, 255, 255, 255]),
            90,
            y_artist as i32,
            info_font_size,
            &lato_bold,
            &artist_up,
        );
        let y_title = 600.0 - 62.0;
        draw_text_mut(
            &mut canvas,
            image::Rgba([255, 255, 255, 255]),
            90,
            y_title as i32,
            info_font_size,
            &lato_bold,
            &format!("\"{}\"", &title_up),
        );
    } else {
        // Draw card_info
        let y_card = 600.0 - 91.27;
        draw_text_mut(
            &mut canvas,
            image::Rgba([255, 255, 255, 255]),
            90,
            y_card as i32,
            info_font_size,
            &lato_bold,
            &card_info,
        );
    }

    // 4. Draw each line of caption
    let mut bars = textwrap::wrap(caption, 34);
    let mut bar_font_size: f32 = 52.0;
    let mut bar_height: u32 = 68;
    let mut bar_gap: u32 = 22;
    let mut last_bar_y = 405;
    let bar_x_offset = 90;
    let bar_x_pad = 10;

    if bars.len() > 4 {
        bars = textwrap::wrap(caption, 46);
        bar_height = 51;
        bar_font_size = 40.0;
        bar_gap = 11;
        // y of the top of the last bar
        last_bar_y = 425;
    }
    let num_of_bars = bars.len();
    let text_descent = lato_semibold.as_scaled(bar_font_size).descent() as i32;

    for (i, line) in bars.iter().enumerate() {
        let y: i32 = last_bar_y - ((num_of_bars - i - 1) as i32 * (bar_height + bar_gap) as i32);
        let bar_size = text_size(bar_font_size, &lato_bold, &line);

        // draw white bar
        draw_filled_rect_mut(
            &mut canvas,
            Rect::at(bar_x_offset, y).of_size(bar_size.0 + 2 * bar_x_pad, bar_height),
            image::Rgba([255, 255, 255, 255]),
        );

        // draw text
        draw_text_mut(
            &mut canvas,
            image::Rgba([0, 0, 0, 255]),
            bar_x_offset + bar_x_pad as i32,
            y - text_descent,
            bar_font_size,
            &lato_semibold,
            &line,
        );
    }

    // 5. Overlay quote image
    let quote_img = image::load_from_memory(QUOTE_SYMBOL)?.resize(38, 38, imageops::Lanczos3);

    let quote_y = last_bar_y - ((num_of_bars - 1) as i32 * (bar_height + bar_gap) as i32);
    imageops::overlay(&mut canvas, &quote_img, 25, quote_y as i64);

    let filename: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 30);
    let filename = format!("{filename}.jpg");
    canvas.save(&filename)?;

    Ok(filename)
}
