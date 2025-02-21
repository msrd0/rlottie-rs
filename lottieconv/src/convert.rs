use rgb::{RGBA8, alt::BGRA8};
use std::convert::Infallible;

pub trait Convert {
	type Out;
	type Err;

	fn convert_frame(&self, from: &[BGRA8], to: &mut [RGBA8]);

	fn add_frame(&mut self, data: &mut [u8]) -> Result<(), Self::Err>;

	fn finish(self) -> Result<Self::Out, Self::Err>;
}

pub enum DummyConvert {}

impl Convert for DummyConvert {
	type Out = ();
	type Err = Infallible;

	fn convert_frame(&self, _: &[BGRA8], _: &mut [RGBA8]) {
		unreachable!()
	}

	fn add_frame(&mut self, _: &mut [u8]) -> Result<(), Self::Err> {
		unreachable!()
	}

	fn finish(self) -> Result<Self::Out, Self::Err> {
		unreachable!()
	}
}
