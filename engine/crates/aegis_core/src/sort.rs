//! Godot's `Array.sort_custom` (core/templates/sort_array.h): introsort, then a final insertion sort; at 16
//! elements or fewer only the insertion sort runs (stable). Ported as is so equal keys land where Godot puts them.

const INTROSORT_THRESHOLD: usize = 16;

/// Sorts `v` with `less(a, b)` ("a goes before b"), exactly like Godot's SortArray.
pub fn godot_sort<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], less: F) {
    let n = v.len();
    if n == 0 {
        return;
    }
    introsort(v, 0, n, &less, bitlog(n) * 2);
    final_insertion_sort(v, 0, n, &less);
}

fn bitlog(mut n: usize) -> usize {
    let mut k = 0;
    while n != 1 {
        n >>= 1;
        k += 1;
    }
    k
}

fn median_of_3<T: Clone, F: Fn(&T, &T) -> bool>(a: &T, b: &T, c: &T, less: &F) -> T {
    if less(a, b) {
        if less(b, c) {
            b.clone()
        } else if less(a, c) {
            c.clone()
        } else {
            a.clone()
        }
    } else if less(a, c) {
        a.clone()
    } else if less(b, c) {
        c.clone()
    } else {
        b.clone()
    }
}

fn partitioner<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], mut first: usize, mut last: usize, pivot: &T, less: &F) -> usize {
    loop {
        while less(&v[first], pivot) {
            first += 1;
        }
        last -= 1;
        while less(pivot, &v[last]) {
            last -= 1;
        }
        if first >= last {
            return first;
        }
        v.swap(first, last);
        first += 1;
    }
}

fn introsort<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], first: usize, mut last: usize, less: &F, mut depth: usize) {
    while last - first > INTROSORT_THRESHOLD {
        if depth == 0 {
            partial_sort(v, first, last, last, less);
            return;
        }
        depth -= 1;
        let pivot = median_of_3(&v[first], &v[first + (last - first) / 2], &v[last - 1], less);
        let cut = partitioner(v, first, last, &pivot, less);
        introsort(v, cut, last, less, depth);
        last = cut;
    }
}

fn push_heap<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], first: usize, mut hole: usize, top: usize, value: T, less: &F) {
    let half = |h: usize| if h == 0 { 0 } else { (h - 1) / 2 }; // C++ (h - 1) / 2 truncates -1 / 2 to 0
    let mut parent = half(hole);
    while hole > top && less(&v[first + parent], &value) {
        v[first + hole] = v[first + parent].clone();
        hole = parent;
        parent = half(hole);
    }
    v[first + hole] = value;
}

fn adjust_heap<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], first: usize, mut hole: usize, len: usize, value: T, less: &F) {
    let top = hole;
    let mut second = 2 * hole + 2;
    while second < len {
        if less(&v[first + second], &v[first + second - 1]) {
            second -= 1;
        }
        v[first + hole] = v[first + second].clone();
        hole = second;
        second = 2 * (second + 1);
    }
    if second == len {
        v[first + hole] = v[first + second - 1].clone();
        hole = second - 1;
    }
    push_heap(v, first, hole, top, value, less);
}

fn pop_heap_to<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, result: usize, value: T, less: &F) {
    v[result] = v[first].clone();
    adjust_heap(v, first, 0, last - first, value, less);
}

fn make_heap<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, less: &F) {
    if last - first < 2 {
        return;
    }
    let len = last - first;
    let mut parent = (len - 2) / 2;
    loop {
        let value = v[first + parent].clone();
        adjust_heap(v, first, parent, len, value, less);
        if parent == 0 {
            return;
        }
        parent -= 1;
    }
}

fn partial_sort<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, middle: usize, less: &F) {
    make_heap(v, first, middle, less);
    for i in middle..last {
        if less(&v[i], &v[first]) {
            let value = v[i].clone();
            pop_heap_to(v, first, middle, i, value, less);
        }
    }
    let mut end = middle;
    while end - first > 1 {
        let value = v[end - 1].clone();
        pop_heap_to(v, first, end - 1, end - 1, value, less);
        end -= 1;
    }
}

fn unguarded_linear_insert<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], mut last: usize, value: T, less: &F) {
    let mut next = last - 1;
    while less(&value, &v[next]) {
        v[last] = v[next].clone();
        last = next;
        next -= 1;
    }
    v[last] = value;
}

fn linear_insert<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, less: &F) {
    let value = v[last].clone();
    if less(&value, &v[first]) {
        let mut i = last;
        while i > first {
            v[i] = v[i - 1].clone();
            i -= 1;
        }
        v[first] = value;
    } else {
        unguarded_linear_insert(v, last, value, less);
    }
}

fn insertion_sort<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, less: &F) {
    if first == last {
        return;
    }
    for i in first + 1..last {
        linear_insert(v, first, i, less);
    }
}

fn final_insertion_sort<T: Clone, F: Fn(&T, &T) -> bool>(v: &mut [T], first: usize, last: usize, less: &F) {
    if last - first > INTROSORT_THRESHOLD {
        insertion_sort(v, first, first + INTROSORT_THRESHOLD, less);
        for i in first + INTROSORT_THRESHOLD..last {
            let value = v[i].clone();
            unguarded_linear_insert(v, i, value, less);
        }
    } else {
        insertion_sort(v, first, last, less);
    }
}

#[cfg(test)]
mod tests {
    use super::godot_sort;

    #[test]
    fn small_sorts_are_stable() {
        let mut v = vec![(1, 'a'), (0, 'b'), (1, 'c'), (0, 'd')];
        godot_sort(&mut v, |a, b| a.0 < b.0);
        assert_eq!(v, vec![(0, 'b'), (0, 'd'), (1, 'a'), (1, 'c')]);
    }

    #[test]
    fn large_sorts_are_sorted() {
        let mut v: Vec<i32> = (0..200).map(|i| (i * 7919 + 13) % 101).collect();
        godot_sort(&mut v, |a, b| a < b);
        assert!(v.windows(2).all(|w| w[0] <= w[1]));
    }
}
