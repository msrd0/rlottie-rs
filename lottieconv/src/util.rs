macro_rules! auto_vectorize {
	(
		pub(crate) fn $ident:ident($($arg_ident:ident : $arg_ty:ty),*) $(-> $ret:ty)? {
			$($body:tt)*
		}
	) => {
		pub(crate) fn $ident($($arg_ident: $arg_ty),*) $(-> $ret)? {
			#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
			#[target_feature(enable = "avx2")]
			#[target_feature(enable = "bmi1")]
			#[target_feature(enable = "bmi2")]
			#[allow(unused_unsafe)]
			unsafe fn avx2($($arg_ident: $arg_ty),*) $(-> $ret)? {
				$($body)*
			}

			#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
			#[target_feature(enable = "sse4.1")]
			#[allow(unused_unsafe)]
			unsafe fn sse4_1($($arg_ident: $arg_ty),*) $(-> $ret)? {
				$($body)*
			}

			fn fallback($($arg_ident: $arg_ty),*) $(-> $ret)? {
				$($body)*
			}

			#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
			if is_x86_feature_detected!("avx2") {
				return unsafe { avx2($($arg_ident),*) };
			}

			#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
			if is_x86_feature_detected!("sse4.1") {
				return unsafe { sse4_1($($arg_ident),*) };
			}

			fallback($($arg_ident),*)
		}
	};
}
