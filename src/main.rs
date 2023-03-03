use cellular_textures::KdTree;
use png::EncodingError;

use std::{fmt::Display, fs::File, io::BufWriter, str::FromStr};

// based on https://blackpawn.com/texts/cellular/default.html
fn main() {
    let rng = fastrand::Rng::with_seed(0);
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        eprintln!("Usage: {} PIXELS CELLS", args[0]);
        eprintln!("Example: {} 3000x3000 10", args[0]);
        std::process::exit(1);
    }

    let bounds =
        parse_pair::<usize>(&args[1], 'x').expect(&format!("Unexpected dimensions: {}", &args[1]));
    let cell_count =
        usize::from_str(&args[2]).expect(&format!("Unexpected cell counts: {}", &args[2]));

    let mut cells: Vec<(usize, usize)> = Vec::new();
    for _ in 0..cell_count {
        cells.push((rng.usize(0..bounds.0), rng.usize(0..bounds.1)));
    }
    let mut pixels: Vec<u8> = vec![0; bounds.0 * bounds.1];

    render(&bounds, &mut pixels, cells);
    write_image("output.png", &mut pixels, bounds).unwrap()
}

fn find_nearest(p: (usize, usize), tree: &KdTree, bounds: &(usize, usize)) -> f64 {
    let mirrors = [
        (p.0, p.1),
        (p.0, bounds.1 - p.1),
        (bounds.0 - p.0, p.1),
        (bounds.0 - p.0, bounds.1 - p.1),
    ];
    mirrors
        .iter()
        .map(|p| tree.mindist(*p))
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .0
}

fn render(bounds: &(usize, usize), pixels: &mut [u8], cells: Vec<(usize, usize)>) {
    let mut distance: Vec<f64> = vec![0.0; (bounds.0 * bounds.1) as usize];
    let tree = KdTree::new(cells.to_vec());
    for i in 0..bounds.0 {
        for j in 0..bounds.1 {
            distance[(i * bounds.1) + j] = find_nearest((i, j), &tree, bounds);
        }
    }
    let maxdist = distance.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
    for i in 0..bounds.0 {
        for j in 0..bounds.1 {
            let index = (i * bounds.1) + j;
            pixels[index] = (u8::MAX as f64 * distance[index] / maxdist) as u8;
        }
    }
}

fn parse_pair<T: FromStr>(s: &str, seperator: char) -> Option<(T, T)> {
    match s.find(seperator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(a), Ok(b)) => Some((a, b)),
            _ => None,
        },
    }
}

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), EncodingError> {
    let file = File::create(filename).unwrap();
    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, bounds.0 as u32, bounds.1 as u32);
    encoder.set_color(png::ColorType::Grayscale);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(pixels)?;
    Ok(())
}

#[allow(dead_code)]
fn print_matrix<T: Display>(arr: &[T], r: usize, c: usize) {
    let mut line = String::new();
    for i in 0..r {
        for j in 0..c {
            line.push_str(&format!("{}|", arr[i * c + j]));
        }
        println!("{}", line);
        line.clear();
    }
}

#[test]
fn test_print_matrix() {
    print_matrix(&vec![1, 2, 3, 4], 2, 2);
}
