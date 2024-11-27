/// Match and downcast a generic type to one of several base types
///
/// ```
/// # use tank_game_core::rules::infrastructure::ecs::Attribute;
/// # use tank_game_core::match_type;
/// # #[derive(Debug)]
/// struct DummyAttribute;
/// # impl Attribute for DummyAttribute {}
/// #
/// let value = DummyAttribute;
/// let attribute_value: &dyn Attribute = &value;
/// match_type!(attribute_value, {
///     value: DummyAttribute => { /* This branch will be called */ }
/// });
/// ```
#[macro_export]
macro_rules! match_type {
    ($any_var:ident, { $( $var_name:ident: $type:ty => $code:expr ),+ }) => {
        {
            use as_any::Downcast;
            use $crate::basic_error;

            let type_id = $any_var.type_id();
            let mut value_handled = false;

            $(
                if type_id == std::any::TypeId::of::<$type>() {
                    value_handled = true;

                    match $any_var.downcast_ref::<$type>() {
                        Some($var_name) => $code,
                        None => panic!("The TypeId of {} matched {} but failed to downcast", stringify!($any_var), stringify!($type)),
                    }
                }
            )+

            if !value_handled {
                Err(basic_error!("No case found for type: {:?}", type_id))
            } else {
                Ok(())
            }
        }
    };
}
