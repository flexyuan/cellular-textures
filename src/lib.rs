#![feature(test)]

extern crate test;

use std::fmt::Debug;
pub type Point = (usize, usize);
pub type DistancePoint = (f64, Point);

pub fn sqr_distance(p1: &Point, p2: &Point) -> f64 {
    let dx = p1.0.abs_diff(p2.0) as f64;
    let dy = p1.1.abs_diff(p2.1) as f64;
    dx.powi(2) + dy.powi(2)
}

#[derive(Debug)]
enum NewNode<T> {
    Branch(Box<NewNode<T>>, Box<NewNode<T>>, T),
    Leafs(Vec<T>),
}

#[derive(Debug)]
pub struct KdTree {
    root: NewNode<Point>,
}

impl KdTree {
    pub fn mindist(self: &Self, p: &Point) -> (f64, Point) {
        let result = nodedistance(&self.root, &p, 0);
        (result.0.sqrt(), result.1)
    }

    pub fn new(xs: Vec<(usize, usize)>) -> KdTree {
        let root = Self::build(xs, 0);
        KdTree { root }
    }

    fn build(xs: Vec<Point>, depth: u32) -> NewNode<Point> {
        if xs.len() < 15 {
            NewNode::Leafs(xs)
        } else {
            let (left, right, median) = split_at_median(xs, depth);
            let left = Self::build(left, depth + 1);
            let right = Self::build(right, depth + 1);
            NewNode::Branch(Box::new(left), Box::new(right), median)
        }
    }
}
fn split_at_median(xs: Vec<Point>, depth: u32) -> (Vec<Point>, Vec<Point>, Point) {
    let comparator = if depth % 2 == 0 {
        |x: &Point| x.0
    } else {
        |x: &Point| x.1
    };
    let mut xs = xs.clone();
    xs.sort_by_key(comparator);
    let median_index = (xs.len() - 1) / 2;
    let mut left: Vec<Point> = Vec::with_capacity(median_index);
    let mut right: Vec<Point> = Vec::with_capacity(xs.len() - median_index - 1);
    for i in 0..median_index {
        left.push(xs[i]);
    }
    for i in (median_index + 1)..xs.len() {
        right.push(xs[i]);
    }
    (left, right, xs[median_index])
}

fn nodedistance(node: &NewNode<Point>, p: &Point, depth: u32) -> (f64, Point) {
    match node {
        NewNode::Leafs(ls) => {
            let key = ls
                .iter()
                .min_by(|p1, p2| {
                    sqr_distance(*p1, p)
                        .partial_cmp(&sqr_distance(*p2, p))
                        .unwrap()
                })
                .unwrap();
            return (sqr_distance(key, p), *key);
        }
        NewNode::Branch(left, right, value) => {
            let pdist = (sqr_distance(value, p), *value);

            let helper = |p1: &NewNode<Point>, p2: &NewNode<Point>| {
                let p1_dist = nodedistance(p1, p, depth + 1);
                let ortho_dist = if depth % 2 == 0 {
                    value.0.abs_diff(p.0)
                } else {
                    value.1.abs_diff(p.1)
                };
                let sm = smaller(p1_dist, pdist);
                if (ortho_dist as f64) < sm.0 {
                    smaller(nodedistance(p2, p, depth + 1), sm)
                } else {
                    sm
                }
            };

            fn smaller(a: DistancePoint, b: DistancePoint) -> DistancePoint {
                if a.0 < b.0 {
                    a
                } else {
                    b
                }
            }

            if depth % 2 == 0 {
                if value.0 > p.0 {
                    helper(left, right)
                } else {
                    helper(right, left)
                }
            } else {
                if value.1 > p.1 {
                    helper(left, right)
                } else {
                    helper(right, left)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use test::Bencher;
    #[test]
    fn test_build() {
        let a = vec![(5, 4), (2, 6), (13, 3), (3, 1), (10, 2), (8, 7)];
        let tree = KdTree::new(a);
        println!("{:?}", tree);
        println!("{:?}", tree.mindist(&(9, 4)));
    }

    #[test]
    fn test_build2() {
        let rng = fastrand::Rng::with_seed(100);
        let mut cells = Vec::new();
        for _ in 0..1000 {
            cells.push((rng.usize(0..1000), rng.usize(0..1000)));
        }
        let tree = KdTree::new(cells.clone());

        for i in 0..1000 {
            for j in 0..1000 {
                let p = (i, j);
                let tree_dist = tree.mindist(&p);
                let (brute_dist, minpos) = find_nearest(p, &cells);
                if tree_dist.0 != brute_dist {
                    panic!("failed for {:?}. Correct pos: {:?}", p, minpos);
                }
            }
        }
    }

    fn find_nearest(p: (usize, usize), cells: &[(usize, usize)]) -> (f64, Point) {
        let mut mindist = sqr_distance(&cells[0], &p);
        let mut min_pos = cells[0];
        for i in cells {
            let k = sqr_distance(i, &p);
            if k < mindist {
                mindist = k;
                min_pos = *i;
            }
        }
        (mindist.sqrt(), min_pos)
    }

    #[bench]
    fn bench_kdtree(b: &mut Bencher) {
        let rng = fastrand::Rng::with_seed(100);
        let mut cells = Vec::new();
        for _ in 0..1000 {
            cells.push((rng.usize(0..1000), rng.usize(0..1000)));
        }
        let tree = KdTree::new(cells);
        let points = (0..100)
            .map(|_| (rng.usize(0..1000), rng.usize(0..1000)))
            .collect::<Vec<Point>>();
        b.iter(|| {
            for p in &points {
                let dummy = tree.mindist(p);
                test::black_box(dummy);
            }
        });
    }

    #[bench]
    fn bench_bruteforce(b: &mut Bencher) {
        let rng = fastrand::Rng::with_seed(100);
        let mut cells = Vec::new();
        for _ in 0..1000 {
            cells.push((rng.usize(0..1000), rng.usize(0..1000)));
        }
        let points = Rc::new(
            (0..100)
                .map(|_| (rng.usize(0..1000), rng.usize(0..1000)))
                .collect::<Vec<Point>>(),
        );
        b.iter(|| {
            points.as_ref().iter().for_each(|p| {
                let dummy = find_nearest(*p, &cells);
                test::black_box(dummy);
            });
        });
    }
}
