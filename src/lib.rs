use std::fmt::Debug;
pub type Point = (usize, usize);

pub fn sqr_distance(p1: Point, p2: Point) -> f64 {
    let dx = p1.0.abs_diff(p2.0) as f64;
    let dy = p1.1.abs_diff(p2.1) as f64;
    dx.powi(2) + dy.powi(2)
}

#[derive(Debug)]
struct Node<T> {
    value: T,
    children: Vec<Node<T>>,
}

#[derive(Debug)]
pub struct KdTree {
    root: Node<Point>,
}

impl KdTree {
    pub fn mindist(self: &Self, p: Point) -> (f64, Point) {
        let result = Self::ndistance(&self.root, p, 0);
        (result.0.sqrt(), result.1)
    }

    fn ndistance(node: &Node<Point>, p: Point, depth: u32) -> (f64, Point) {
        let pdist = (sqr_distance(node.value, p), node.value);
        let helper = |p1: &Node<Point>, p2: &Node<Point>| {
            let p1_dist = KdTree::ndistance(p1, p, depth + 1);
            let ortho_dist = if depth % 2 == 0 {
                node.value.0.abs_diff(p.0)
            } else {
                node.value.1.abs_diff(p.1)
            };
            let sm = smaller(p1_dist, pdist);
            if (ortho_dist as f64) <= sm.0 {
                smaller(KdTree::ndistance(p2, p, depth + 1), sm)
            } else {
                sm
            }
        };

        fn smaller(a: (f64, Point), b: (f64, Point)) -> (f64, Point) {
            if a.0 < b.0 {
                a
            } else {
                b
            }
        }

        match &node.children[..] {
            [v1] => smaller(Self::ndistance(v1, p, depth + 1), pdist),
            [] => pdist,
            [v1, v2] => {
                if depth % 2 == 0 {
                    if node.value.0 > p.0 {
                        helper(v1, v2)
                    } else {
                        helper(v2, v1)
                    }
                } else {
                    if node.value.1 > p.1 {
                        helper(v1, v2)
                    } else {
                        helper(v2, v1)
                    }
                }
            }
            _ => panic!("Found more than two children"),
        }
    }

    pub fn new(xs: Vec<(usize, usize)>) -> KdTree {
        let root = Self::build(xs, 0);
        KdTree { root }
    }

    fn build(xs: Vec<Point>, depth: u32) -> Node<Point> {
        let (left, right, median) = Self::split_at_median(xs, depth);
        let mut children = Vec::new();
        if left.len() > 0 {
            children.push(Self::build(left, depth + 1));
        }
        if right.len() > 0 {
            children.push(Self::build(right, depth + 1));
        }
        Node {
            value: median,
            children,
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
}

#[test]
fn basic_test() {
    let _a = Node {
        value: (32, 32),
        children: vec![],
    };
}

#[test]
fn test_build() {
    let a = vec![(5, 4), (2, 6), (13, 3), (3, 1), (10, 2), (8, 7)];
    let tree = KdTree::new(a);
    println!("{:?}", tree);
    println!("{:?}", tree.mindist((9, 4)));
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
            let tree_dist = tree.mindist(p);
            let (brute_dist, minpos) = find_nearest(p, &cells);
            if tree_dist.0 != brute_dist {
                panic!("failed for {:?}. Correct pos: {:?}", p, minpos);
            }
        }
    }
    fn find_nearest(p: (usize, usize), cells: &[(usize, usize)]) -> (f64, Point) {
        let mut mindist = sqr_distance(cells[0], p);
        let mut min_pos = cells[0];
        for i in cells {
            let k = sqr_distance(*i, p);
            if k < mindist {
                mindist = k;
                min_pos = *i;
            }
        }
        (mindist.sqrt(), min_pos)
    }
}