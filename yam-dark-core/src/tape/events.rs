use yam_common::ScalarType;

pub trait EventListener<'de> {
    type ScalarValue;

    fn on_scalar(&mut self, scalar_value: Self::ScalarValue, _scalar_type: ScalarType);
}
