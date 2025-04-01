use clap::Parser;
use std::fs;
use html5ever::{parse_document, tendril::TendrilSink, ParseOpts};
use markup5ever_rcdom::RcDom;
use extract_article::{HtmlWalker, SimpleSelectorParser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args  {
    /// url of the website to extract article
    url: String,

    /// css selector to find article
    #[arg(short, long, default_value = "body")]
    selector: String,

    /// define it to download the article to a file
    #[arg(short, long, default_value_t = false)]
    download: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let selector = SimpleSelectorParser::new(&args.selector).parse()
        .expect(format!("Could not parse css selector '{}'", args.selector).as_str());
    let resp = reqwest::get(args.url).await.unwrap();
    let data = resp.text().await.unwrap();
    let mut opts: ParseOpts = Default::default();
    opts.tree_builder.scripting_enabled = false;
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut data.as_bytes())
        .unwrap();
    let mut buffer = String::new();
    let mut walker = HtmlWalker::new(&dom.document, &mut buffer, selector);
    if !args.download {
        //no need to look into head elements
        walker.ignore_elements.insert("head".to_string());
    }
    walker.walk();
    if args.download {
        let filename = if walker.title.is_empty() {
            String::from("/tmp/article.txt")
        } else {
            String::from("/tmp/") + &walker.title + ".txt"
        };
        fs::write(&filename, buffer).expect(format!("Unable to write file '{}'", &filename).as_str());
    } else {
        println!("{}", buffer);
    }
    Ok(())
}
