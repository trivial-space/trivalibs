use crate::{binding::ValueBinding, Painter};

#[derive(Clone, Copy)]
pub struct SamplerProps {
	pub address_mode_u: wgpu::AddressMode,
	pub address_mode_v: wgpu::AddressMode,
	pub mag_filter: wgpu::FilterMode,
	pub min_filter: wgpu::FilterMode,
	pub mipmap_filter: wgpu::FilterMode,
	pub sample_depth: bool,
}

impl Default for SamplerProps {
	fn default() -> Self {
		Self::NEAREST
	}
}

impl SamplerProps {
	pub const NEAREST: SamplerProps = SamplerProps {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Nearest,
		min_filter: wgpu::FilterMode::Nearest,
		mipmap_filter: wgpu::FilterMode::Nearest,
		sample_depth: false,
	};

	pub const LINEAR: SamplerProps = SamplerProps {
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Linear,
		mipmap_filter: wgpu::FilterMode::Nearest,
		sample_depth: false,
	};
}

#[derive(Clone, Copy)]
pub struct Sampler(pub(crate) usize);

impl Sampler {
	pub fn create(painter: &mut Painter, props: SamplerProps) -> Self {
		let sampler = painter.device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: props.address_mode_u,
			address_mode_v: props.address_mode_v,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: props.mag_filter,
			min_filter: props.min_filter,
			mipmap_filter: props.mipmap_filter,
			compare: props.sample_depth.then(|| wgpu::CompareFunction::LessEqual),
			..Default::default()
		});

		painter.samplers.push(sampler);

		Self(painter.samplers.len() - 1)
	}

	pub fn binding(&self) -> ValueBinding {
		ValueBinding::Sampler(*self)
	}
}

pub struct SamplerBuilder<'a> {
	props: SamplerProps,
	painter: &'a mut Painter,
}

impl<'a> SamplerBuilder<'a> {
	pub fn new(painter: &'a mut Painter) -> Self {
		Self {
			props: SamplerProps::default(),
			painter,
		}
	}

	pub fn create(self) -> Sampler {
		Sampler::create(self.painter, self.props)
	}

	pub fn with_address_mode_u(mut self, mode: wgpu::AddressMode) -> Self {
		self.props.address_mode_u = mode;
		self
	}

	pub fn with_address_mode_v(mut self, mode: wgpu::AddressMode) -> Self {
		self.props.address_mode_v = mode;
		self
	}

	pub fn with_address_modes(mut self, mode: wgpu::AddressMode) -> Self {
		self.props.address_mode_u = mode;
		self.props.address_mode_v = mode;
		self
	}

	pub fn with_mag_filter(mut self, filter: wgpu::FilterMode) -> Self {
		self.props.mag_filter = filter;
		self
	}

	pub fn with_min_filter(mut self, filter: wgpu::FilterMode) -> Self {
		self.props.min_filter = filter;
		self
	}

	pub fn with_filters(mut self, filter: wgpu::FilterMode) -> Self {
		self.props.mag_filter = filter;
		self.props.min_filter = filter;
		self
	}

	pub fn with_mipmap_filter(mut self, filter: wgpu::FilterMode) -> Self {
		self.props.mipmap_filter = filter;
		self
	}

	pub fn with_depth_sampling(mut self) -> Self {
		self.props.sample_depth = true;
		self
	}
}
