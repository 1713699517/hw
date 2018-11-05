use png::HasParameters;
use std::{
    fs::File,
    io::{BufWriter, Read},
    path::{Path, PathBuf}
};
use structopt::StructOpt;

use integral_geometry::{Point, Rect, Size};
use landgen::{
    outline_template::OutlineTemplate,
    template_based::TemplatedLandGenerator,
    LandGenerationParameters,
    LandGenerator
};
use mapgen::MapGenerator;
use lfprng::LaggedFibonacciPRNG;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(short = "s", long = "seed", default_value = "TEST_SEED")]
    seed: String,
    #[structopt(short = "d", long = "dump-before-distort")]
    dump_before_distort: bool,
    #[structopt(short = "b", long = "dump-before-bezierize")]
    dump_before_bezierize: bool,
    #[structopt(short = "f", long = "distance-divisor", default_value = "100")]
    distance_divisor: u32,
    #[structopt(short = "i", long = "templates-file")]
    templates_file: Option<String>,
    #[structopt(short = "t", long = "template-type")]
    template_type: Option<String>
}

fn template() -> OutlineTemplate {
    let mut template = OutlineTemplate::new(Size::new(4096, 2048));
    template.islands = vec![vec![
        Rect::new(100, 2050, 1, 1),
        Rect::new(100, 500, 400, 1200),
        Rect::new(3600, 500, 400, 1200),
        Rect::new(3900, 2050, 1, 1),
    ]];
    template.fill_points = vec![Point::new(2047, 2047)];

    template
}

fn dump(
    template: &OutlineTemplate,
    seed: &[u8],
    distance_divisor: u32,
    skip_distort: bool,
    skip_bezier: bool,
    file_name: &Path,
) -> std::io::Result<()> {
    let params = LandGenerationParameters::new(0 as u8, 255, distance_divisor, skip_distort, skip_bezier);
    let landgen = TemplatedLandGenerator::new(template.clone());
    let mut prng = LaggedFibonacciPRNG::new(seed);
    let land = landgen.generate_land(&params, &mut prng);

    let file = File::create(file_name)?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, land.width() as u32, land.height() as u32); // Width is 2 pixels and height is 1.
    encoder
        .set(png::ColorType::Grayscale)
        .set(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;

    writer.write_image_data(land.raw_pixels()).unwrap();

    Ok(())
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let template =
        if let Some(path) = opt.templates_file {
            let mut result = String::new();
            File::open(path)
                .expect("Unable to read templates file")
                .read_to_string(&mut result);

            let mut generator = MapGenerator::new();

            let bom = b"\xEF\xBB\xBF";
            let source = if &result.as_bytes()[..bom.len()] == &bom[..] {
                &result[bom.len()..]
            } else {
                &result[..]
            };

            generator.import_yaml_templates(source);

            let template_type = &opt.template_type
                .expect("No template type specified");
            generator.get_template(template_type)
                .expect(&format!("Template type {} not found", template_type))
                .clone()
        } else {
            template()
        };

    if opt.dump_before_distort {
        dump(
            &template,
            opt.seed.as_str().as_bytes(),
            opt.distance_divisor,
            true,
            true,
            Path::new("out.before_distort.png"),
        )
        .unwrap();
    }
    if opt.dump_before_bezierize {
        dump(
            &template,
            opt.seed.as_str().as_bytes(),
            opt.distance_divisor,
            false,
            true,
            Path::new("out.before_bezier.png"),
        )
        .unwrap();
    }
    dump(
        &template,
        opt.seed.as_str().as_bytes(),
        opt.distance_divisor,
        false,
        false,
        Path::new("out.full.png"),
    )
    .unwrap();
}
