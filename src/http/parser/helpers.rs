/// validates the `seq`uence using `predicate`
///
/// returns the consumed position and a boolean that indicates if minimum attempts is fulfilled
pub fn consume(seq: &[u8], predicate: fn(seq: &[u8]) -> (usize, bool), min: usize) -> (usize, bool) {
    let mut i = 0;
    let mut attempts = 0;
    while i < seq.len() {
        let (pos, valid) = predicate(&seq[i..]);
        i += pos;
        if !valid {
            break;
        }
        attempts += 1;
    }
    return (i, attempts > min);
}
