use std::collections::VecDeque;

static START: (usize, usize) = (0, 3);
static END: (usize, usize) = (3, 0);
static TARGET: i64 = 30;

static MAZE: &[&[&str]] = &[
    &["*", "8", "-", "1"],
    &["4", "*", "11", "*"],
    &["+", "4", "-", "18"],
    &[".", "-", "9", "*"],
];

fn bfs() {
    let mut queue = VecDeque::new();

    let cur = START;
    let orb = 22;
    let op = None;
    let moves = vec![];
    queue.push_back((cur, orb, op, moves));

    while !queue.is_empty() {
        let (cur, mut orb, mut op, moves) = queue.pop_front().unwrap();
        println!("BFS: {:?}, orb={}", cur, orb);

        let x = cur.0;
        let y = cur.1;

        if cur != START {
            let tile = MAZE[y][x];
            match tile {
                "+" | "-" | "*" => {
                    assert!(op.is_none());
                    op = Some(tile);
                }
                n => {
                    let n: i64 = n.parse().unwrap();
                    match op.unwrap() {
                        "+" => orb += n,
                        "*" => orb *= n,
                        "-" => orb -= n,
                        _ => panic!(),
                    }
                    op = None;
                }
            }
        }

        if cur == END {
            if orb == TARGET {
                println!("we got there!");
                println!("{:#?}", moves);
                return;
            } else {
                continue;
            }
        }

        // try to move in all 4 directions
        let nx = x + 1;
        let ny = y;
        println!("trying {},{}", nx, ny);
        if nx < 4 && (nx, ny) != START {
            let mut moves = moves.clone();
            moves.push("east");
            queue.push_back(((nx, ny), orb, op, moves));
        }

        let nx = x.checked_sub(1);
        let ny = y;
        if let Some(nx) = nx {
            println!("trying {},{}", nx, ny);
            if (nx, ny) != START {
                let mut moves = moves.clone();
                moves.push("west");
                queue.push_back(((nx, ny), orb, op, moves));
            }
        }

        let nx = x;
        let ny = y.checked_sub(1);
        if let Some(ny) = ny {
            println!("trying {},{}", nx, ny);
            if (nx, ny) != START {
                let mut moves = moves.clone();
                moves.push("north");
                queue.push_back(((nx, ny), orb, op, moves));
            }
        }

        let nx = x;
        let ny = y + 1;
        println!("trying {},{}", nx, ny);
        if ny < 4 && (nx, ny) != START {
            let mut moves = moves.clone();
            moves.push("south");
            queue.push_back(((nx, ny), orb, op, moves));
        }

        println!("queue size = {}", queue.len());
    }
}

fn main() {
    bfs();
}
