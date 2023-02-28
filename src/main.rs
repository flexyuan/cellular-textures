use png::EncodingError;

use std::{fmt::Display, fs::File, io::BufWriter, str::FromStr};

// based on https://blackpawn.com/texts/cellular/default.html
fn main() {
    let rng = fastrand::Rng::with_seed(0);
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        eprintln!("Usage: {} PIXELS CELLS", args[0]);
        eprintln!("Example: {} 3000*3000 10", args[0]);
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

    render(&bounds, &mut pixels, &cells);
    write_image("output.png", &mut pixels, bounds).unwrap()
}

fn wrap_dist(a: &(usize, usize), b: &(usize, usize), bounds: &(usize, usize)) -> f64 {
    let mut dx = a.0.abs_diff(b.0) as f64;
    let mut dy = a.1.abs_diff(b.1) as f64;
    let width = bounds.0 as f64;
    let height = bounds.0 as f64;
    if dx > width / 2.0 {
        dx = width - dx;
    }
    if dy > height / 2.0 {
        dy = height - dy;
    }
    (dx.powi(2) + dy.powi(2)).sqrt()
}

fn find_nearest((r, c): (usize, usize), cells: &[(usize, usize)], bounds: &(usize, usize)) -> f64 {
    let mut mindist = wrap_dist(&cells[0], &(r, c), bounds);
    for i in cells {
        let k = wrap_dist(&i, &(r, c), bounds);
        if k < mindist {
            mindist = k;
        }
    }
    mindist
}

fn render(bounds: &(usize, usize), pixels: &mut [u8], cells: &[(usize, usize)]) {
    let mut distance: Vec<f64> = vec![0.0; (bounds.0 * bounds.1) as usize];
    for i in 0..bounds.0 {
        for j in 0..bounds.1 {
            distance[(i * bounds.1) + j] = find_nearest((i, j), cells, &bounds);
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
