use crate::tape::Mark;
use yam_common::ScalarType;

pub trait EventListener {
    fn on_scalar(&mut self, value: &[u8], _scalar_type: ScalarType) -> Mark;
    fn on_scalar_continued(&mut self, value: &[u8], _scalar_type: ScalarType) -> Mark;
}
