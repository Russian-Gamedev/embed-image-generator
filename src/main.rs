use handlebars::{handlebars_helper, Handlebars};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use usvg::NodeExt;

handlebars_helper!(teaser_dy: |v: i64| format!("{}", v * 40));

#[derive(Serialize, Deserialize)]
struct Embed {
    #[serde(rename = "jam-info-bg-width")]
    bg_width: String,
    #[serde(rename = "jam-info-bg-height")]
    bg_height: String,
    #[serde(rename = "jam-info-y")]
    info_y: String,
    #[serde(rename = "jam-info")]
    info: String,
    #[serde(rename = "jam-title")]
    title: String,
    #[serde(rename = "jam-teaser")]
    teaser: Vec<String>,
    #[serde(rename = "jam-season")]
    season: String,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage:\n\tminimal <in-svg> <out-png>");
        return;
    }

    let mut opt = usvg::Options::default();
    opt.resources_dir = std::fs::canonicalize(&args[1])
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    opt.fontdb.load_fonts_dir("./assets/fonts");
    opt.font_family = String::from("Mulish");

    let template_data = std::fs::read(&args[1]).unwrap();

    let mut embed_data = Embed {
        bg_width: "458".to_string(),
        bg_height: "70".to_string(),
        info_y: "83".to_string(),
        info: "3 дня и 17 игр".to_string(),
        title: "Wizard Jam".to_string(),
        teaser: vec![
            "У админов наступила ностальгия новогодних джемов,".to_string(),
            "пусть вьюга станет нашим путеводом!".to_string(),
        ],
        season: "Зима 2020".to_string(),
    };

    let mut reg = Handlebars::new();
    reg.register_helper("dy", Box::new(teaser_dy));

    let mut data = reg
        .render_template(
            std::str::from_utf8(template_data.as_slice()).expect("Found invalid UTF-8"),
            &serde_json::json!(embed_data),
        )
        .expect("Failed to render template");

    let rtree = usvg::Tree::from_data(&data.as_bytes(), &opt.to_ref()).unwrap();
    let info = rtree.node_by_id("jam-info").unwrap();
    let ibbox = info.calculate_bbox().unwrap();

    // MAGIC NUMBERS
    embed_data.bg_width = (ibbox.width() + 53.0).to_string();
    embed_data.bg_height = (50.0 + 20.0).to_string();
    println!("{:}", embed_data.bg_height);
    embed_data.info_y = (48.0 + 10.0 + 40.0).to_string();

    data = reg
        .render_template(
            std::str::from_utf8(template_data.as_slice()).expect("Found invalid UTF-8"),
            &serde_json::json!(embed_data),
        )
        .expect("Failed to render template");

    let tree = usvg::Tree::from_data(&data.as_bytes(), &opt.to_ref()).unwrap();
    let pixmap_size = tree.svg_node().size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&tree, usvg::FitTo::Original, pixmap.as_mut()).unwrap();
    pixmap.save_png(&args[2]).unwrap();
}
