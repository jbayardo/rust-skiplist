#[allow_internal_unsafe]
macro_rules! unlikely {
  ($e:expr) => {
    #[allow(unused_unsafe)]
    unsafe {
      std::intrinsics::unlikely($e)
    }
  };
}

#[allow_internal_unsafe]
macro_rules! likely {
  ($e:expr) => {
    #[allow(unused_unsafe)]
    unsafe {
      std::intrinsics::likely($e)
    }
  };
}
