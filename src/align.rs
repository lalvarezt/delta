use std::cmp::max;
use std::collections::VecDeque;

const DELETION_COST: usize = 2;
const INSERTION_COST: usize = 2;
// extra cost for starting a new group of changed tokens
const INITIAL_MISMATCH_PENALTY: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    NoOp,
    Deletion,
    Insertion,
}

use Operation::*;

/// Needleman-Wunsch / Wagner-Fischer table for computation of edit distance and associated
/// alignment.
#[derive(Debug)]
pub struct Alignment<'a> {
    pub x: Vec<&'a str>,
    pub y: Vec<&'a str>,
    table: Vec<Operation>,
    cost: usize,
    dim: [usize; 2],
}

impl<'a> Alignment<'a> {
    /// Fill table for Levenshtein distance / alignment computation
    pub fn new(x: Vec<&'a str>, y: Vec<&'a str>) -> Self {
        // TODO: Something downstream of the alignment algorithm requires that the first token in
        // both x and y is "", so this is explicitly inserted in `tokenize()`.
        let dim = [y.len() + 1, x.len() + 1];
        let table = vec![NoOp; dim[0] * dim[1]];
        let mut alignment = Self {
            x,
            y,
            table,
            cost: 0,
            dim,
        };
        alignment.fill();
        alignment
    }

    /// Fill table for Levenshtein distance / alignment computation
    pub fn fill(&mut self) {
        // x is written along the top of the table; y is written down the left side of the
        // table. Also, we insert a 0 in cell (0, 0) of the table, so x and y are shifted by one
        // position. Therefore, the element corresponding to (x[i], y[j]) is in column (i + 1) and
        // row (j + 1); the index of this element is given by index(i, j).
        // Traceback needs one operation per cell. Costs are needed only for the
        // current and preceding rows, which share this buffer.
        let mut costs = vec![0; self.dim[1]];
        for (i, cost) in costs.iter_mut().enumerate().skip(1) {
            self.table[i] = Deletion;
            *cost = i * DELETION_COST + INITIAL_MISMATCH_PENALTY;
        }
        for j in 1..self.dim[0] {
            self.table[j * self.dim[1]] = Insertion;
        }

        // Visit adjacent cells in the row-major table. Both the preceding row and
        // the preceding column are complete when a cell is evaluated.
        for (j, y_j) in self.y.iter().enumerate() {
            let mut diagonal_cost = costs[0];
            costs[0] = (j + 1) * INSERTION_COST + INITIAL_MISMATCH_PENALTY;
            for (i, x_i) in self.x.iter().enumerate() {
                let (left, up) = (self.index(i, j + 1), self.index(i + 1, j));
                // The order of the candidates matters if two of them have the
                // same cost as in that case we choose the first one. Consider
                // insertions and deletions before matches in order to group
                // changes together. Insertions are preferred to deletions in
                // order to highlight moved tokens as a deletion followed by an
                // insertion (as the edit sequence is read backwards we need to
                // choose the insertion first)
                let mismatch_cost = |cost, operation, basic_cost| {
                    cost + basic_cost
                        + if operation == NoOp {
                            INITIAL_MISMATCH_PENALTY
                        } else {
                            0
                        }
                };
                let candidates = [
                    (
                        Insertion,
                        mismatch_cost(costs[i + 1], self.table[up], INSERTION_COST),
                    ),
                    (
                        Deletion,
                        mismatch_cost(costs[i], self.table[left], DELETION_COST),
                    ),
                    (
                        NoOp,
                        if x_i == y_j {
                            diagonal_cost
                        } else {
                            usize::MAX
                        },
                    ),
                ];
                let index = self.index(i + 1, j + 1);
                let &(operation, cost) = candidates.iter().min_by_key(|(_, cost)| *cost).unwrap();
                self.table[index] = operation;
                diagonal_cost = costs[i + 1];
                costs[i + 1] = cost;
            }
        }
        self.cost = costs[self.x.len()];
    }

    /// Read edit operations from the table.
    pub fn operations(&self) -> Vec<Operation> {
        let mut ops = VecDeque::with_capacity(max(self.x.len(), self.y.len()));
        let (mut i, mut j) = (self.x.len(), self.y.len());
        loop {
            let operation = self.table[self.index(i, j)];
            ops.push_front(operation);
            // The initial row and column terminate traceback, including (0, 0).
            if i == 0 || j == 0 {
                break;
            }
            match operation {
                Insertion => j -= 1,
                Deletion => i -= 1,
                NoOp => {
                    i -= 1;
                    j -= 1;
                }
            }
            if i == 0 && j == 0 {
                break;
            }
        }
        Vec::from(ops)
    }

    pub fn coalesced_operations(&self) -> Vec<(Operation, usize)> {
        run_length_encode(self.operations())
    }

    // Row-major storage of 2D array.
    fn index(&self, i: usize, j: usize) -> usize {
        j * self.dim[1] + i
    }
}

fn run_length_encode<T>(sequence: Vec<T>) -> Vec<(T, usize)>
where
    T: Copy,
    T: PartialEq,
{
    let mut encoded = Vec::with_capacity(sequence.len());

    if sequence.is_empty() {
        return encoded;
    }

    let end = sequence.len();
    let (mut i, mut j) = (0, 1);
    let mut curr = &sequence[i];
    loop {
        if j == end || sequence[j] != *curr {
            encoded.push((*curr, j - i));
            if j == end {
                return encoded;
            } else {
                curr = &sequence[j];
                i = j;
            }
        }
        j += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn test_alignment_matches_full_cost_table() {
        // Include empty inputs, repeated tokens, and equal-cost paths. Traceback
        // tie-breaking determines which words receive emphasis in the output.
        fn reference(x: &[&str], y: &[&str]) -> (usize, Vec<Operation>) {
            let width = x.len() + 1;
            let mut table = vec![(0, NoOp, 0); width * (y.len() + 1)];
            for (i, cell) in table.iter_mut().enumerate().take(width).skip(1) {
                *cell = (i * DELETION_COST + INITIAL_MISMATCH_PENALTY, Deletion, 0);
            }
            for j in 1..=y.len() {
                table[j * width] = (j * INSERTION_COST + INITIAL_MISMATCH_PENALTY, Insertion, 0);
            }
            for (i, a) in x.iter().enumerate() {
                for (j, b) in y.iter().enumerate() {
                    let (left, up, diagonal) =
                        (i + (j + 1) * width, i + 1 + j * width, i + j * width);
                    let mismatch = |parent: usize, cost| {
                        table[parent].0
                            + cost
                            + if table[parent].1 == NoOp {
                                INITIAL_MISMATCH_PENALTY
                            } else {
                                0
                            }
                    };
                    table[i + 1 + (j + 1) * width] = *[
                        (mismatch(up, INSERTION_COST), Insertion, up),
                        (mismatch(left, DELETION_COST), Deletion, left),
                        (
                            if a == b {
                                table[diagonal].0
                            } else {
                                usize::MAX
                            },
                            NoOp,
                            diagonal,
                        ),
                    ]
                    .iter()
                    .min_by_key(|cell| cell.0)
                    .unwrap();
                }
            }
            let mut index = table.len() - 1;
            let cost = table[index].0;
            let mut operations = Vec::new();
            loop {
                let (_, operation, parent) = table[index];
                operations.push(operation);
                if parent == 0 {
                    break;
                }
                index = parent;
            }
            operations.reverse();
            (cost, operations)
        }

        let mut inputs = vec![Vec::new()];
        let mut level = vec![Vec::new()];
        for _ in 0..4 {
            level = level
                .iter()
                .flat_map(|tokens| {
                    ["", "a", "b"].iter().map(move |token| {
                        let mut next = tokens.clone();
                        next.push(*token);
                        next
                    })
                })
                .collect();
            inputs.extend(level.clone());
        }
        for x in &inputs {
            for y in &inputs {
                let alignment = Alignment::new(x.clone(), y.clone());
                assert_eq!(
                    (alignment.cost, alignment.operations()),
                    reference(x, y),
                    "{x:?} -> {y:?}"
                );
            }
        }
    }

    #[test]
    fn test_run_length_encode() {
        assert_eq!(run_length_encode::<usize>(vec![]), vec![]);
        assert_eq!(run_length_encode(vec![0]), vec![(0, 1)]);
        assert_eq!(run_length_encode(vec!["0", "0"]), vec![("0", 2)]);
        assert_eq!(
            run_length_encode(vec![0, 0, 1, 2, 2, 2, 3, 4, 4, 4]),
            vec![(0, 2), (1, 1), (2, 3), (3, 1), (4, 3)]
        );
    }

    #[test]
    fn test_0() {
        TestCase {
            before: "aaa",
            after: "aba",
            distance: 5,
            parts: (2, 4),
            operations: vec![NoOp, Deletion, Insertion, NoOp],
        }
        .run();
    }

    #[test]
    fn test_0_nonascii() {
        TestCase {
            before: "ááb",
            after: "áaa",
            distance: 9,
            parts: (4, 5),
            operations: vec![NoOp, Deletion, Deletion, Insertion, Insertion],
        }
        .run();
    }

    #[test]
    fn test_1() {
        TestCase {
            before: "kitten",
            after: "sitting",
            distance: 13,
            parts: (5, 9),
            operations: vec![
                Deletion,  // K -
                Insertion, // - S
                NoOp,      // I I
                NoOp,      // T T
                NoOp,      // T T
                Deletion,  // E -
                Insertion, // - I
                NoOp,      // N N
                Insertion, // - G
            ],
        }
        .run();
    }

    #[test]
    fn test_2() {
        TestCase {
            before: "saturday",
            after: "sunday",
            distance: 10,
            parts: (4, 9),
            operations: vec![
                NoOp,      // S S
                Deletion,  // A -
                Deletion,  // T -
                NoOp,      // U U
                Deletion,  // R -
                Insertion, // - N
                NoOp,      // D D
                NoOp,      // A A
                NoOp,      // Y Y
            ],
        }
        .run();
    }

    #[test]
    fn test_3() {
        TestCase {
            // Prefer [Deletion NoOp Insertion] over [Insertion NoOp Deletion]
            before: "ab",
            after: "ba",
            distance: 6,
            parts: (2, 3),
            operations: vec![
                Deletion,  // a -
                NoOp,      // b b
                Insertion, // - a
            ],
        }
        .run();
    }

    #[test]
    fn test_4() {
        // Deletions are grouped together.
        TestCase {
            before: "AABB",
            after: "AB",
            distance: 5,
            parts: (2, 4),
            operations: vec![
                NoOp,     // A A
                Deletion, // A -
                Deletion, // B -
                NoOp,     // B B
            ],
        }
        .run();
    }

    #[test]
    fn test_5() {
        // Insertions are grouped together.
        TestCase {
            before: "AB",
            after: "AABB",
            distance: 5,
            parts: (2, 4),
            operations: vec![
                NoOp,      // A A
                Insertion, // - A
                Insertion, // - B
                NoOp,      // B B
            ],
        }
        .run();
    }

    #[test]
    fn test_6() {
        // Insertion and Deletion are grouped together.
        TestCase {
            before: "AAABBB",
            after: "ACB",
            distance: 11,
            parts: (5, 7),
            operations: vec![
                NoOp,      // A A
                Deletion,  // A -
                Deletion,  // A -
                Deletion,  // B -
                Deletion,  // B -
                Insertion, // - C
                NoOp,      // B B
            ],
        }
        .run();
    }

    struct TestCase<'a> {
        before: &'a str,
        after: &'a str,
        distance: usize,
        parts: (usize, usize),
        operations: Vec<Operation>,
    }

    impl<'a> TestCase<'a> {
        pub fn run(&self) {
            self.assert_string_distance_parts();
            assert_eq!(operations(self.before, self.after), self.operations);
        }

        fn assert_string_distance_parts(&self) {
            self.assert_string_levenshtein_distance();
            assert_eq!(string_distance_parts(self.before, self.after), self.parts);
            assert_eq!(string_distance_parts(self.after, self.before), self.parts);
        }

        fn assert_string_levenshtein_distance(&self) {
            assert_eq!(
                string_levenshtein_distance(self.before, self.after),
                self.distance
            );
            assert_eq!(
                string_levenshtein_distance(self.after, self.before),
                self.distance
            );
        }
    }

    fn string_distance_parts(x: &str, y: &str) -> (usize, usize) {
        let (x, y) = (
            x.graphemes(true).collect::<Vec<&str>>(),
            y.graphemes(true).collect::<Vec<&str>>(),
        );
        Alignment::new(x, y).distance_parts()
    }

    fn string_levenshtein_distance(x: &str, y: &str) -> usize {
        let (x, y) = (
            x.graphemes(true).collect::<Vec<&str>>(),
            y.graphemes(true).collect::<Vec<&str>>(),
        );
        Alignment::new(x, y).levenshtein_distance()
    }

    fn operations<'a>(x: &'a str, y: &'a str) -> Vec<Operation> {
        let (x, y) = (
            x.graphemes(true).collect::<Vec<&str>>(),
            y.graphemes(true).collect::<Vec<&str>>(),
        );
        Alignment::new(x, y).operations()
    }

    impl<'a> Alignment<'a> {
        pub fn distance_parts(&self) -> (usize, usize) {
            let (mut numer, mut denom) = (0, 0);
            for op in self.operations() {
                if op != NoOp {
                    numer += 1;
                }
                denom += 1;
            }
            (numer, denom)
        }

        /// Compute levenshtein distance from the filled table.
        pub fn levenshtein_distance(&self) -> usize {
            self.cost
        }
    }
}
