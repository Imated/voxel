#[macro_export]
macro_rules! fatal {
    ($($arg:tt)+) =>  {
        {
            $crate::error!($($arg)+);
            abort()
        }
    }
}
