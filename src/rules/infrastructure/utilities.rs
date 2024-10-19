#[macro_export]
macro_rules! match_type {
    ($any_var:ident, { $( $var_name:ident: $type:ty => $code:expr ),+ }) => {
        {
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
                Err(Box::new($crate::rules::infrastructure::error::RuleError::Generic(format!("No case found for type: {:?}", type_id))))
            } else {
                Ok(())
            }
        }
    };
}
