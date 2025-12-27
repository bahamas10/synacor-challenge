use std::collections::HashMap;

type Cache = HashMap<(u16, u16), u16>;

fn fn6049(mut r0: u16, mut r1: u16, r7: u16, cache: &mut Cache) -> u16 {
    if r0 == 0 {
        r0 = (r1 + 1) % 32768;
        return r0;
    }

    if r1 == 0 {
        r0 = (r0 + 32767) % 32768; // dec by 1
        r1 = r7;
        return if let Some(v) = cache.get(&(r0, r1)) {
            *v
        } else {
            let v = fn6049(r0, r1, r7, cache);
            cache.insert((r0, r1), v);
            v
        };
    }

    let tmp = r0;
    r1 = (r1 + 32767) % 32768; // decrement by 1
    r0 = if let Some(v) = cache.get(&(r0, r1)) {
        *v
    } else {
        let v = fn6049(r0, r1, r7, cache);
        cache.insert((r0, r1), v);
        v
    };

    r1 = r0;
    r0 = tmp;

    r0 = (r0 + 32767) % 32768; // decrement by 1

    if let Some(v) = cache.get(&(r0, r1)) {
        *v
    } else {
        let v = fn6049(r0, r1, r7, cache);
        cache.insert((r0, r1), v);
        v
    }
}

fn main() {
    for i in 1..32768 {
        let mut cache = Cache::new();
        println!("trying to solve r7={}", i);
        let value = fn6049(4, 1, i, &mut cache);
        if value == 6 {
            println!("it worked!");
            eprintln!("{} worked!", i);
        }
    }
}
