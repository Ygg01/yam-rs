pub trait EventListener {
    type ScalarValue;

    fn on_scalar(&mut self, scalar_value: Self::ScalarValue);
}
