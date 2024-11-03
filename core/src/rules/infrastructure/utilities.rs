/// Match and downcast a generic type to one of several base types
///
/// ```
/// # use tank_game::rules::infrastructure::ecs::{Handle, AttributeValue};
/// # use tank_game::match_type;
/// let value: u32 = 3;
/// let attribute_value: &dyn AttributeValue = &value;
/// match_type!(attribute_value, {
///     value: u32 => assert_eq!(*value, 3),
///     _unused: Handle => panic!("This branch won't be called")
/// });
/// ```
#[macro_export]
macro_rules! match_type {
    ($any_var:ident, { $( $var_name:ident: $type:ty => $code:expr ),+ }) => {
        {
            use as_any::Downcast;

            let type_id = $any_var.type_id();
            let mut value_handled = false;

            $(
                if type_id == std::any::TypeId::of::<$type>() {
                    value_handled = true;

                    match $any_var.as_ref().downcast_ref::<$type>() {
                        Some($var_name) => $code,
                        None => panic!("The TypeId of {} matched {} but failed to downcast", stringify!($any_var), stringify!($type)),
                    }
                }
            )+

            if !value_handled {
                Err(Box::new($crate::rules::infrastructure::RuleError::Generic(format!("No case found for type: {:?}", type_id))))
            } else {
                Ok(())
            }
        }
    };
}
