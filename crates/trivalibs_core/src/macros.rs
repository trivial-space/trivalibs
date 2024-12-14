
macro_rules! attribute_alias {(
	$(
			#[apply($name:ident $(!)?)] = $( #[$($attrs:tt)*] )+;
	)*
) => (
	$(
			$crate::ඞ_with_dollar! {( $_:tt ) => (
					// Let's not do the paste + module + re-export dance here since it
					// is less likely for an attribute name to collide with a prelude item.
					#[allow(nonstandard_style)]
					#[macro_export]
					macro_rules! $name {( $_($item:tt)* ) => (
					$( #[$($attrs)*] )+
							$_($item)*
					)}
					#[allow(unused_imports)]
					pub use $name;
			)}
	)*
)}

#[doc(hidden)]
/** Not part of the public API*/
#[macro_export]
macro_rules! ඞ_with_dollar {( $($rules:tt)* ) => (
	macro_rules! __emit__ { $($rules)* }
	__emit__! { $ }
)}

attribute_alias! {
	#[apply(gpu_data)] =
	#[repr(C)]
	#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)];
}

pub use ::macro_rules_attribute::apply;

#[macro_export]
macro_rules! hashmap {
	() => {
			::std::collections::HashMap::new()
	};

	($($key:expr => $value:expr),+ $(,)?) => {
			::std::collections::HashMap::from([ $(($key, $value)),* ])
	};
}

#[macro_export]
macro_rules! bmap {
	() => {
			::std::collections::BTreeMap::new()
	};

	($($key:expr => $value:expr),+ $(,)?) => {
			::std::collections::BTreeMap::from([ $(($key, $value)),* ])
	};
}
