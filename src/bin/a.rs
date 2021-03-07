use proconio::input;
use std::fmt;

const W: i32 = 10000;
fn main() {
    input! {
        n: usize,
        xys: [(i32, i32, i32); n],
    }
    let ps = xys.iter().map(|&(x, y, _)| (x, y)).collect::<Vec<_>>();
    let size = xys.iter().map(|&(_, _, s)| s).collect::<Vec<_>>();
    let input = Input { n, ps, size };
    let mut out = input
        .ps
        .iter()
        .map(|&(x, y)| Rect {
            x1: x,
            y1: y,
            x2: x + 1,
            y2: y + 1,
        })
        .collect::<Vec<Rect>>();

    solve(&input, &mut out);
    let mut score = compute_score(&input, &out);
    local_search(&input, &mut out, score);
    score = compute_score(&input, &out);

    // 答えを出力
    for rect in out.iter() {
        println!("{}", rect);
    }
    eprintln!("{}", score);
}

fn solve(input: &Input, out: &mut Vec<Rect>) {
    let mut order = (0..input.n).collect::<Vec<usize>>();
    order.sort_by_cached_key(|&k| {
        (input.ps[k].0)
            .min(9999 - input.ps[k].0)
            .min(input.ps[k].1)
            .min(9999 - input.ps[k].1)
    });
    for i in order.into_iter() {
        // 各企業iの上下左右におけるスペース増加可能分を調べる
        let ex_spaces = vec![out[i].x1, out[i].y1, 10000 - out[i].x2, 10000 - out[i].y2];

        // 増加可能分が小さい方向から順番に
        // r_iを超えない範囲でスペースを増加させる(下は各方向の番号)
        //     1
        //   0 x 2
        //     3
        let mut d_order = vec![0, 1, 2, 3];
        d_order.sort_by_cached_key(|&k| ex_spaces[k]);
        for d in d_order.into_iter() {
            let mut ex_len = 0;
            let mut ng = ex_spaces[d] + 1;
            while ng - ex_len > 1 {
                let mid = (ex_len + ng) / 2;
                match d {
                    0 => out[i].x1 -= mid,
                    1 => out[i].y1 -= mid,
                    2 => out[i].x2 += mid,
                    3 => out[i].y2 += mid,
                    _ => (),
                };
                if out[i].size() > input.size[i]
                    || (0..input.n)
                        .into_iter()
                        .any(|j| i != j && intersect(&out[i], &out[j]))
                {
                    ng = mid;
                } else {
                    ex_len = mid;
                }
                match d {
                    0 => out[i].x1 += mid,
                    1 => out[i].y1 += mid,
                    2 => out[i].x2 -= mid,
                    3 => out[i].y2 -= mid,
                    _ => (),
                };
            }
            match d {
                0 => out[i].x1 -= ex_len,
                1 => out[i].y1 -= ex_len,
                2 => out[i].x2 += ex_len,
                3 => out[i].y2 += ex_len,
                _ => (),
            };
        }
    }
}

fn local_search(input: &Input, out: &mut Vec<Rect>, score: i64) {
    // 変形する長方形を決める
    // 一つ長方形を選ぶより、tmpの値でソートしたベクタを作る方がよさそう
    let mut rect_i = 0;
    let mut now = 256;
    for i in 0..input.n {
        let val = if out[i].size() > input.size[i] {
            1.0 - input.size[i] as f64 / out[i].size() as f64 / 2.0
        } else {
            out[i].size() as f64 / input.size[i] as f64 / 2.0
        };
        let tmp = ((-(2.0 * std::f64::consts::PI * val).cos() / 2.0 + 0.5) * 255.0) as i32;
        // tmp が 255に近いほど要求面積に近い
        if val >= 0.5 {
            // r_iよりも大きく赤くなる
            continue;
        } else {
            // r_iよりも小さく青くなる
            if tmp < now {
                now = tmp;
                rect_i = i;
            }
        }
    }
    // 変形方向を決める 4方向
    // (下は各方向の番号)
    //     1
    //   0 x 2
    //     3
    // 4方向のうち最もスコアがよい方向に変形させる
    // スコアは差分計算で求めたい
    let mut score = score;
    let mut ex_len = -1;
    let mut shrs = vec![];
    let mut real_d = -1;
    for d in 0..4 {
        let (now_score, now_ex_len, now_shrs) = modify(&input, out, rect_i, d);
        if score < now_score {
            score = now_score;
            ex_len = now_ex_len;
            shrs = now_shrs;
            real_d = d;
        }
    }
    if ex_len != -1 {
        match real_d {
            0 => out[rect_i].x1 -= ex_len,
            1 => out[rect_i].y1 -= ex_len,
            2 => out[rect_i].x2 += ex_len,
            3 => out[rect_i].y2 += ex_len,
            _ => (),
        };
        for (j, shr_d, shr_len) in shrs.iter() {
            match shr_d {
                0 => out[*j].x2 -= shr_len,
                1 => out[*j].y2 -= shr_len,
                2 => out[*j].x1 += shr_len,
                3 => out[*j].y1 += shr_len,
                _ => (),
            };
        }
    }
}

fn modify(
    input: &Input,
    out: &mut Vec<Rect>,
    rect_i: usize,
    d: i32,
) -> (i64, i32, Vec<(usize, i32, i32)>) {
    // r_i を超えないように二分探索
    // 条件：
    //    (x_j,y_j)があったらng
    let ex_space = match d {
        0 => out[rect_i].x1,
        1 => out[rect_i].y1,
        2 => 10000 - out[rect_i].x2,
        3 => 10000 - out[rect_i].y2,
        _ => 0,
    };
    let mut ex_len = 0;
    let mut ng = ex_space + 1;
    while ng - ex_len > 1 {
        let mid = (ex_len + ng) / 2;
        match d {
            0 => out[rect_i].x1 -= mid,
            1 => out[rect_i].y1 -= mid,
            2 => out[rect_i].x2 += mid,
            3 => out[rect_i].y2 += mid,
            _ => (),
        };
        if out[rect_i].size() > input.size[rect_i]
            || (0..input.n)
                .into_iter()
                .any(|j| rect_i != j && out[rect_i].contain_key(&input.ps[j]))
        {
            ng = mid;
        } else {
            ex_len = mid;
        }
        match d {
            0 => out[rect_i].x1 += mid,
            1 => out[rect_i].y1 += mid,
            2 => out[rect_i].x2 -= mid,
            3 => out[rect_i].y2 -= mid,
            _ => (),
        };
    }
    match d {
        0 => out[rect_i].x1 -= ex_len,
        1 => out[rect_i].y1 -= ex_len,
        2 => out[rect_i].x2 += ex_len,
        3 => out[rect_i].y2 += ex_len,
        _ => (),
    };

    // intersectしてるやつを縮める
    // あとで戻せるように、縮めた長方形は覚えておく
    let mut shrinkings = vec![];
    for j in 0..input.n {
        if rect_i == j || !intersect(&out[rect_i], &out[j]) {
            continue;
        }
        // 縮める方向を決める
        // 交差しなくなり、かつ縮める量が一番少ないやつに
        // 0 :jのx2を、out[rect_i].x1に
        // 1 :jのy2を、out[rect_i].y1に
        // 2 :jのx1を、out[rect_i].x2に
        // 3 :jのy1を、out[rect_i].y2に
        let mut loss_area = i32::max_value();
        let mut real_d = -1;
        for now_d in 0..4 {
            let mut loss_now = i32::max_value();
            match now_d {
                0 => {
                    loss_now = if out[j].x1 < out[rect_i].x1 && input.ps[j].0 <= out[rect_i].x1 {
                        (out[j].x2 - out[rect_i].x1) * (out[j].y2 - out[j].y1)
                    } else {
                        i32::max_value()
                    };
                }
                1 => {
                    loss_now = if out[j].y1 < out[rect_i].y1 && input.ps[j].1 <= out[rect_i].y1 {
                        (out[j].x2 - out[j].x1) * (out[j].y2 - out[rect_i].y1)
                    } else {
                        i32::max_value()
                    };
                }
                2 => {
                    loss_now = if out[rect_i].x2 < out[j].x2 && out[rect_i].x2 <= input.ps[j].0 {
                        (out[rect_i].x2 - out[j].x1) * (out[j].y2 - out[j].y1)
                    } else {
                        i32::max_value()
                    };
                }
                3 => {
                    loss_now = if out[rect_i].y2 < out[j].y2 && out[rect_i].y2 <= input.ps[j].1 {
                        (out[j].x2 - out[j].x1) * (out[rect_i].y2 - out[j].y1)
                    } else {
                        i32::max_value()
                    };
                }
                _ => (),
            };
            if loss_area > loss_now {
                loss_area = loss_now;
                real_d = now_d;
            }
        }
        match real_d {
            0 => {
                shrinkings.push((j, real_d, out[j].x2 - out[rect_i].x1));
                out[j].x2 = out[rect_i].x1;
            }
            1 => {
                shrinkings.push((j, real_d, out[j].y2 - out[rect_i].y1));
                out[j].y2 = out[rect_i].y1;
            }
            2 => {
                shrinkings.push((j, real_d, out[rect_i].x2 - out[j].x1));
                out[j].x1 = out[rect_i].x2;
            }
            3 => {
                shrinkings.push((j, real_d, out[rect_i].y2 - out[j].y1));
                out[j].y1 = out[rect_i].y2;
            }
            _ => (),
        };
        if intersect(&out[rect_i], &out[j]) {
            panic!();
        }
    }
    let score = compute_score(input, out);

    // 変形を戻す

    match d {
        0 => out[rect_i].x1 += ex_len,
        1 => out[rect_i].y1 += ex_len,
        2 => out[rect_i].x2 -= ex_len,
        3 => out[rect_i].y2 -= ex_len,
        _ => (),
    };

    for (j, shr_d, shr_len) in shrinkings.iter() {
        match shr_d {
            0 => out[*j].x2 += shr_len,
            1 => out[*j].y2 += shr_len,
            2 => out[*j].x1 -= shr_len,
            3 => out[*j].y1 -= shr_len,
            _ => (),
        };
    }

    // d方向に縮めたときの縮める量ex_lenとそのときのスコアscoreを返す
    (score, ex_len, shrinkings)
}

struct Input {
    n: usize,
    ps: Vec<(i32, i32)>,
    size: Vec<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}", self.x1, self.y1, self.x2, self.y2)
    }
}

impl Rect {
    fn size(&self) -> i32 {
        (self.x2 - self.x1) * (self.y2 - self.y1)
    }
    fn contain_key(&self, other: &(i32, i32)) -> bool {
        self.x1 <= other.0 && other.0 <= self.x2 && self.y1 <= other.1 && other.1 <= self.y2
    }
}

fn intersect(r1: &Rect, r2: &Rect) -> bool {
    r1.x2.min(r2.x2) > r1.x1.max(r2.x1) && r1.y2.min(r2.y2) > r1.y1.max(r2.y1)
}

fn compute_score(input: &Input, out: &Vec<Rect>) -> i64 {
    let mut score = 0.0;
    for i in 0..input.n {
        if out[i].x1 < 0 || out[i].x2 > W || out[i].y1 < 0 || out[i].y2 > W {
            eprintln!("rectangle {} is out of range", i);
            return 0;
        }
        if out[i].x1 >= out[i].x2 || out[i].y1 >= out[i].y2 {
            eprintln!("rectangle {} does not have positive area", i);
            return 0;
        }
        if !(out[i].x1 <= input.ps[i].0
            && input.ps[i].0 < out[i].x2
            && out[i].y1 <= input.ps[i].1
            && input.ps[i].1 < out[i].y2)
        {
            eprintln!("rectangle {} does not contain point {}", i, i);
            continue;
        }
        for j in 0..i {
            if intersect(&out[i], &out[j]) {
                eprintln!("rectangles {} and {} overlap", j, i);
                return 0;
            }
        }
        let s = out[i].size().min(input.size[i]) as f64 / out[i].size().max(input.size[i]) as f64;
        score += 1.0 - (1.0 - s) * (1.0 - s);
    }
    (1e9 * score / input.n as f64).round() as i64
}
