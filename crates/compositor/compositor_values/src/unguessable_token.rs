// Source dependency: base/unguessable_token.h (two 64-bit halves; empty is 0).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnguessableToken(pub [u64; 2]);
impl UnguessableToken {
    pub fn is_empty(self) -> bool {
        self.0 == [0, 0]
    }
}
