use chrono::NaiveDate;
use genpdf::{elements, style, Alignment, Element};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
};

#[derive(Debug, Deserialize)]
struct Post {
    title: String,
    content: String,
    date: String,
    images: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image_data: HashMap<String, Vec<u8>> = load_image_data("images.bin")?;
    let mut posts: Vec<Post> = serde_json::from_reader(File::open("backup.json")?)?;
    posts.sort_by_key(|post| NaiveDate::parse_from_str(&post.date, "%A %d %B %Y").ok());

    let font_family =
        genpdf::fonts::from_files("./fonts", "OpenSans", None).expect("Failed to load font family");

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);

    let mut doc = genpdf::Document::new(font_family);
    doc.set_paper_size(genpdf::PaperSize::A4);
    doc.set_page_decorator(decorator);

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

        let title_style = style::Style::new().with_font_size(24).bold();
        let title_element = elements::Paragraph::new(post.title.clone().replace("&amp;", "&"))
            .aligned(Alignment::Left)
            .styled(title_style);
        doc.push(title_element);

        let date_style = style::Style::new().with_font_size(12).bold();
        let date_element = elements::Paragraph::new(post.date.clone())
            .aligned(Alignment::Left)
            .styled(date_style);
        doc.push(date_element);

        doc.push(elements::Break::new(1));
        let content_lines = post.content.lines();
        for line in content_lines {
            let content_element = elements::Paragraph::new(line.replace("&amp;", "&").to_string());
            doc.push(content_element);
        }

        for image_url in &post.images {
            match image_data.get(image_url) {
                Some(image_data) => {
                    let image_data = image::load_from_memory(&image_data)?;
                    if image_data.color().has_alpha() {
                        continue; // images with alpha channel not supported
                    }
                    let image = genpdf::elements::Image::from_dynamic_image(image_data)?
                        .with_alignment(genpdf::Alignment::Center);

                    doc.push(image);
                }
                None => {
                    bar.set_message(format!("Downloading image for: {}", post.title));
                    match download_image(image_url) {
                        Ok(image_data) => {
                            let loaded_image = image::load_from_memory(&image_data)?;
                            if loaded_image.color().has_alpha() {
                                continue; // images with alpha channel not supported
                            }
                            let image = genpdf::elements::Image::from_dynamic_image(loaded_image)?
                                .with_alignment(genpdf::Alignment::Center);
                            doc.push(image);
                        }
                        Err(e) => {
                            eprintln!("Failed to download image: {}", e);
                        }
                    }
                }
            }
        }

        bar.inc(1);
    }

    bar.finish_with_message("PDF generation complete!");
    println!("Saving PDF, this may take a few minutes...");
    let output_file = File::create("Blog1.pdf")?;
    doc.render(&mut std::io::BufWriter::new(output_file))?;
    println!("All done!");

    Ok(())
}

fn load_image_data(path: &str) -> io::Result<HashMap<String, Vec<u8>>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let decoded: HashMap<String, Vec<u8>> =
        bincode::deserialize(&buffer).expect("Deserialization failed");
    Ok(decoded)
}

fn download_image(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    Ok(response.bytes()?.to_vec())
}
