use clap::Parser;
use dirs;
use extract_article::{HtmlWalker, SimpleSelectorParser};
use html5ever::{parse_document, tendril::TendrilSink, ParseOpts};
use markup5ever_rcdom::RcDom;
use serde::Deserialize;
use serde_json;
use std::fs;

fn strip_trailing_newline(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix("\n"))
        .unwrap_or(input)
}

#[derive(Deserialize, Debug)]
struct ConfigItem {
    url: String,
    selector: String,
}

fn load_config() -> Result<Vec<ConfigItem>, Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;
    let config_path = config_dir.join("extract_article/config.json");

    let config_contents = fs::read_to_string(config_path)?;
    let config: Vec<ConfigItem> = serde_json::from_str(&config_contents)?;

    Ok(config)
}

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
    let mut selector = SimpleSelectorParser::new(&args.selector).parse()
        .expect(format!("Could not parse css selector '{}'", args.selector).as_str());
    if let Ok(config) = load_config() {
        for item in config.iter(){
            if args.url.contains(&item.url)  {
                selector = SimpleSelectorParser::new(&item.selector).parse()
                    .expect(format!("Could not parse css selector '{}'", args.selector).as_str());
            }
        };
    }
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
        //no need to look into head elements to find title
        walker.ignore_elements.insert("head".to_string());
    }
    walker.walk();
    if args.download {
        let filename = if walker.title.is_empty() {
            String::from("/tmp/article.txt")
        } else {
            String::from("/tmp/") + &walker.title + ".txt"
        };
        fs::write(&filename, strip_trailing_newline(&buffer)).expect(format!("Unable to write file '{}'", &filename).as_str());
    } else {
        println!("{}", strip_trailing_newline(&buffer));
    }
    Ok(())
}
