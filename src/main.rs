use indicatif::{ProgressBar, ProgressStyle};
use printpdf::{
    BuiltinFont, ColorBits, ImageTransform, ImageXObject, Mm, PdfDocument, PdfDocumentReference,
    PdfLayerIndex, PdfPageIndex, Px,
};
use reqwest::blocking;
use serde::Deserialize;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Post {
    title: String,
    content: String,
    date: String,
    images: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let posts: Vec<Post> = serde_json::from_reader(File::open("backup.json")?)?;
    let (doc, _, _) = PdfDocument::new(
        "Gnostic Esoteric Study & Work Aids",
        Mm(210.0),
        Mm(297.0),
        "Base Layer",
    );
    let bar = ProgressBar::new(posts.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
            )?
            .progress_chars("#>-"),
    );
    for post in &posts {
        bar.set_message(format!("Processing post: {}", post.title));

        // Create a new page and add content
        let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Post Layer");
        add_text_to_pdf(&doc, page, layer, &post.title, 30, true)?;
        add_text_to_pdf(&doc, page, layer, &post.date, 12, true)?;
        add_text_to_pdf(&doc, page, layer, &post.content, 12, false)?;

        // Download and add images
        for image_url in &post.images {
            bar.set_message(format!("Downloading image for: {}", post.title));
            if let Ok(image_data) = download_image(image_url) {
                add_image_to_pdf(&doc, page, layer, &image_data)?;
            }
        }

        bar.inc(1); // Move progress bar forward
    }

    bar.finish_with_message("PDF generation complete!");

    let output_file = File::create("Blog.pdf")?;
    doc.save(&mut std::io::BufWriter::new(output_file))?;

    Ok(())
}

fn add_text_to_pdf(
    doc: &PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
    text: &str,
    font_size: i64,
    bold: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_layer = doc.get_page(page).get_layer(layer.clone());
    let font = doc.add_builtin_font(if bold {
        BuiltinFont::HelveticaBold
    } else {
        BuiltinFont::Helvetica
    })?;
    current_layer.use_text(text, font_size as f32, Mm(10.0), Mm(10.0), &font);
    Ok(())
}

fn download_image(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = blocking::get(url)?;
    Ok(response.bytes()?.to_vec())
}

fn add_image_to_pdf(
    doc: &PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
    image_data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let image = image::ImageReader::new(std::io::Cursor::new(image_data))
        .with_guessed_format()?
        .decode()?
        .to_rgb8();

    let (img_width, img_height) = image.dimensions();

    // Convert the image into raw pixel data
    let img_data = image.into_raw();

    // Create an ImageXObject for the PDF using the raw pixel data
    let pdf_image = ImageXObject {
        width: Px(img_width as usize),
        height: Px(img_height as usize),
        color_space: printpdf::ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: img_data,
        image_filter: None,
        smask: None,
        clipping_bbox: None,
    };

    let final_image = printpdf::Image::from(pdf_image);

    // Place the image on the specified layer
    let current_layer = doc.get_page(page).get_layer(layer);
    final_image.add_to_layer(current_layer.clone(), ImageTransform::default());

    Ok(())
}
