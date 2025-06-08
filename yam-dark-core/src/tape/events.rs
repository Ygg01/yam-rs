use crate::tape::Mark;
use yam_common::ScalarType;

pub trait EventListener {
    type Value<'a>;
    fn on_scalar(&mut self, value: Self::Value<'_>, _scalar_type: ScalarType, mark: Mark);
    fn on_scalar_continued(&mut self, value: Self::Value<'_>, _scalar_type: ScalarType, mark: Mark);
}
