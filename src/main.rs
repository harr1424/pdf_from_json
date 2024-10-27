use genpdf::{elements, style, Alignment, Element};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::fs::File;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
struct Post {
    title: String,
    content: String,
    date: String,
    // images: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut posts: Vec<Post> = serde_json::from_reader(File::open("backup.json")?)?;
    posts.sort_by_key(|post| {
        NaiveDate::parse_from_str(&post.date, "%A %d %B %Y").ok()
    });
    
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

        let date_style = style::Style::new().with_font_size(12);
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

        // Add images (commented out as per your request)
        // for image_url in &post.images {
        //     bar.set_message(format!("Downloading image for: {}", post.title));
        //     if let Ok(image_data) = download_image(image_url) {
        //         let image = genpdf::elements::Image::from_dynamic_image(image::load_from_memory(&image_data)?)?;
        //         page.push(image);
        //     }
        // }

        bar.inc(1);
    }

    bar.finish_with_message("PDF generation complete!");

    let output_file = File::create("Blog.pdf")?;
    doc.render(&mut std::io::BufWriter::new(output_file))?;

    Ok(())
}

// fn download_image(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
//     let response = reqwest::blocking::get(url)?;
//     Ok(response.bytes()?.to_vec())
// }
