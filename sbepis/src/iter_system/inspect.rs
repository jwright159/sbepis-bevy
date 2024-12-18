use std::borrow::Cow;

use bevy::ecs::archetype::ArchetypeComponentId;
use bevy::ecs::component::{ComponentId, Tick};
use bevy::ecs::query::Access;
use bevy::ecs::schedule::InternedSystemSet;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;

pub trait IntoInspectSystemTrait<In: SystemInput, Out, Marker>:
	IntoSystem<In, Out, Marker>
where
	Out: IntoIterator,
{
	fn iter_inspect<B, BIn, BOut, MarkerB>(self, system: B) -> IntoInspectSystem<Self, B>
	where
		Out: 'static,
		B: IntoSystem<BIn, BOut, MarkerB>,
		for<'a> BIn: SystemInput<Inner<'a> = <Out as IntoIterator>::Item>,
	{
		IntoInspectSystem::new(self, system)
	}
}
impl<T, In, Out, Marker> IntoInspectSystemTrait<In, Out, Marker> for T
where
	T: IntoSystem<In, Out, Marker>,
	In: SystemInput,
	Out: IntoIterator,
{
}

pub struct IntoInspectSystem<A, B> {
	a: A,
	b: B,
}

impl<A, B> IntoInspectSystem<A, B> {
	pub const fn new(a: A, b: B) -> Self {
		Self { a, b }
	}
}

#[doc(hidden)]
pub struct IsInspectSystemMarker;

impl<A, B, IA, OA, IB, OB, MA, MB>
	IntoSystem<IA, Vec<OA::Item>, (IsInspectSystemMarker, OA, OB, IB, MA, MB)>
	for IntoInspectSystem<A, B>
where
	IA: SystemInput,
	OA: IntoIterator,
	<OA as IntoIterator>::Item: Clone,
	A: IntoSystem<IA, OA, MA>,
	B: IntoSystem<IB, OB, MB>,
	for<'a> IB: SystemInput<Inner<'a> = <OA as IntoIterator>::Item>,
{
	type System = InspectSystem<A::System, B::System>;

	fn into_system(this: Self) -> Self::System {
		let system_a = IntoSystem::into_system(this.a);
		let system_b = IntoSystem::into_system(this.b);
		let name = format!("Inspect({}, {})", system_a.name(), system_b.name());
		InspectSystem::new(system_a, system_b, Cow::Owned(name))
	}
}

pub struct InspectSystem<A, B> {
	a: A,
	b: B,
	name: Cow<'static, str>,
	component_access: Access<ComponentId>,
	archetype_component_access: Access<ArchetypeComponentId>,
}

impl<A, B> InspectSystem<A, B> {
	pub const fn new(a: A, b: B, name: Cow<'static, str>) -> Self {
		Self {
			a,
			b,
			name,
			component_access: Access::new(),
			archetype_component_access: Access::new(),
		}
	}
}

impl<A, B> System for InspectSystem<A, B>
where
	A: System,
	<A as System>::Out: IntoIterator,
	// TODO: Now that InRef exists, we don't need Clone.
	// However, I can't figure out how to constrain it, what with lifetimes and all...
	<A::Out as IntoIterator>::Item: Clone,
	B: System,
	for<'a> B::In: SystemInput<Inner<'a> = <A::Out as IntoIterator>::Item>,
{
	type In = A::In;
	type Out = Vec<<A::Out as IntoIterator>::Item>;

	fn name(&self) -> Cow<'static, str> {
		self.name.clone()
	}

	fn component_access(&self) -> &Access<ComponentId> {
		&self.component_access
	}

	fn archetype_component_access(&self) -> &Access<ArchetypeComponentId> {
		&self.archetype_component_access
	}

	fn is_send(&self) -> bool {
		self.a.is_send() && self.b.is_send()
	}

	fn is_exclusive(&self) -> bool {
		self.a.is_exclusive() || self.b.is_exclusive()
	}

	fn has_deferred(&self) -> bool {
		self.a.has_deferred() || self.b.has_deferred()
	}

	unsafe fn run_unsafe(
		&mut self,
		input: SystemIn<'_, Self>,
		world: UnsafeWorldCell,
	) -> Self::Out {
		let value = self.a.run_unsafe(input, world);
		value
			.into_iter()
			.inspect(|x| {
				self.b.run_unsafe(x.clone(), world);
			})
			.collect()
	}

	fn run(&mut self, input: SystemIn<'_, Self>, world: &mut World) -> Self::Out {
		let value = self.a.run(input, world);
		value
			.into_iter()
			.inspect(|x| {
				self.b.run(x.clone(), world);
			})
			.collect()
	}

	#[inline]
	fn apply_deferred(&mut self, world: &mut World) {
		self.a.apply_deferred(world);
		self.b.apply_deferred(world);
	}

	#[inline]
	fn queue_deferred(&mut self, mut world: bevy::ecs::world::DeferredWorld) {
		self.a.queue_deferred(world.reborrow());
		self.b.queue_deferred(world);
	}

	#[inline]
	unsafe fn validate_param_unsafe(&mut self, world: UnsafeWorldCell) -> bool {
		unsafe { self.a.validate_param_unsafe(world) && self.b.validate_param_unsafe(world) }
	}

	fn initialize(&mut self, world: &mut World) {
		self.a.initialize(world);
		self.b.initialize(world);
		self.component_access.extend(self.a.component_access());
		self.component_access.extend(self.b.component_access());
	}

	fn update_archetype_component_access(&mut self, world: UnsafeWorldCell) {
		self.a.update_archetype_component_access(world);
		self.b.update_archetype_component_access(world);

		self.archetype_component_access
			.extend(self.a.archetype_component_access());
		self.archetype_component_access
			.extend(self.b.archetype_component_access());
	}

	fn check_change_tick(&mut self, change_tick: Tick) {
		self.a.check_change_tick(change_tick);
		self.b.check_change_tick(change_tick);
	}

	fn default_system_sets(&self) -> Vec<InternedSystemSet> {
		let mut default_sets = self.a.default_system_sets();
		default_sets.append(&mut self.b.default_system_sets());
		default_sets
	}

	fn get_last_run(&self) -> Tick {
		self.a.get_last_run()
	}

	fn set_last_run(&mut self, last_run: Tick) {
		self.a.set_last_run(last_run);
		self.b.set_last_run(last_run);
	}
}

unsafe impl<A, B> ReadOnlySystem for InspectSystem<A, B>
where
	A: ReadOnlySystem,
	<A as System>::Out: IntoIterator,
	<A::Out as IntoIterator>::Item: Clone,
	B: ReadOnlySystem,
	for<'a> B::In: SystemInput<Inner<'a> = <A::Out as IntoIterator>::Item>,
{
}

impl<A, B> Clone for InspectSystem<A, B>
where
	A: Clone,
	B: Clone,
{
	fn clone(&self) -> Self {
		InspectSystem::new(self.a.clone(), self.b.clone(), self.name.clone())
	}
}
