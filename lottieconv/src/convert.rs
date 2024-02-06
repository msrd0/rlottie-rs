use std::convert::Infallible;

pub trait Convert {
	type Out;
	type Err;

	fn convert_frame(&self, from: &[rlottie::Color], to: &mut [crate::Color]);

	fn add_frame(&mut self, data: &mut [u8]) -> Result<(), Self::Err>;

	fn finish(self) -> Result<Self::Out, Self::Err>;
}

pub enum DummyConvert {}

impl Convert for DummyConvert {
	type Out = ();
	type Err = Infallible;

	fn convert_frame(&self, _: &[rlottie::Color], _: &mut [crate::Color]) {
		unreachable!()
	}

	fn add_frame(&mut self, _: &mut [u8]) -> Result<(), Self::Err> {
		unreachable!()
	}

	fn finish(self) -> Result<Self::Out, Self::Err> {
		unreachable!()
	}
}
