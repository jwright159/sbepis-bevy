use std::borrow::Cow;

use bevy::ecs::archetype::ArchetypeComponentId;
use bevy::ecs::component::{ComponentId, Tick};
use bevy::ecs::query::Access;
use bevy::ecs::schedule::InternedSystemSet;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;

pub trait IntoFlattenSystemTrait<In: SystemInput, Out, Marker>:
	IntoSystem<In, Out, Marker>
where
	Out: IntoIterator,
{
	fn iter_flatten(self) -> IntoFlattenSystem<Self>
	where
		Out: 'static,
	{
		IntoFlattenSystem::new(self)
	}
}
impl<T, In, Out, Marker> IntoFlattenSystemTrait<In, Out, Marker> for T
where
	T: IntoSystem<In, Out, Marker>,
	In: SystemInput,
	Out: IntoIterator,
{
}

pub struct IntoFlattenSystem<A> {
	a: A,
}

impl<A> IntoFlattenSystem<A> {
	pub const fn new(a: A) -> Self {
		Self { a }
	}
}

#[doc(hidden)]
pub struct IsFlattenSystemMarker;

impl<A, IA, OA, MA>
	IntoSystem<
		IA,
		Vec<<<OA as IntoIterator>::Item as IntoIterator>::Item>,
		(IsFlattenSystemMarker, OA, MA),
	> for IntoFlattenSystem<A>
where
	IA: SystemInput,
	OA: IntoIterator,
	<OA as IntoIterator>::Item: IntoIterator,
	A: IntoSystem<IA, OA, MA>,
{
	type System = FlattenSystem<A::System>;

	fn into_system(this: Self) -> Self::System {
		let system_a = IntoSystem::into_system(this.a);
		let name = format!("Flatten({})", system_a.name());
		FlattenSystem::new(system_a, Cow::Owned(name))
	}
}

pub struct FlattenSystem<A> {
	a: A,
	name: Cow<'static, str>,
	component_access: Access<ComponentId>,
	archetype_component_access: Access<ArchetypeComponentId>,
}

impl<A> FlattenSystem<A> {
	pub const fn new(a: A, name: Cow<'static, str>) -> Self {
		Self {
			a,
			name,
			component_access: Access::new(),
			archetype_component_access: Access::new(),
		}
	}
}

impl<A> System for FlattenSystem<A>
where
	A: System,
	<A as System>::Out: IntoIterator,
	<A::Out as IntoIterator>::Item: IntoIterator,
{
	type In = A::In;
	type Out = Vec<<<A::Out as IntoIterator>::Item as IntoIterator>::Item>;

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
		self.a.is_send()
	}

	fn is_exclusive(&self) -> bool {
		self.a.is_exclusive()
	}

	fn has_deferred(&self) -> bool {
		self.a.has_deferred()
	}

	unsafe fn run_unsafe(
		&mut self,
		input: SystemIn<'_, Self>,
		world: UnsafeWorldCell,
	) -> Self::Out {
		let value = self.a.run_unsafe(input, world);
		value.into_iter().flatten().collect()
	}

	fn run(&mut self, input: SystemIn<'_, Self>, world: &mut World) -> Self::Out {
		let value = self.a.run(input, world);
		value.into_iter().flatten().collect()
	}

	#[inline]
	fn apply_deferred(&mut self, world: &mut World) {
		self.a.apply_deferred(world);
	}

	#[inline]
	fn queue_deferred(&mut self, world: bevy::ecs::world::DeferredWorld) {
		self.a.queue_deferred(world);
	}

	#[inline]
	unsafe fn validate_param_unsafe(&mut self, world: UnsafeWorldCell) -> bool {
		unsafe { self.a.validate_param_unsafe(world) }
	}

	fn initialize(&mut self, world: &mut World) {
		self.a.initialize(world);
		self.component_access.extend(self.a.component_access());
	}

	fn update_archetype_component_access(&mut self, world: UnsafeWorldCell) {
		self.a.update_archetype_component_access(world);

		self.archetype_component_access
			.extend(self.a.archetype_component_access());
	}

	fn check_change_tick(&mut self, change_tick: Tick) {
		self.a.check_change_tick(change_tick);
	}

	fn default_system_sets(&self) -> Vec<InternedSystemSet> {
		self.a.default_system_sets()
	}

	fn get_last_run(&self) -> Tick {
		self.a.get_last_run()
	}

	fn set_last_run(&mut self, last_run: Tick) {
		self.a.set_last_run(last_run);
	}
}

unsafe impl<A> ReadOnlySystem for FlattenSystem<A>
where
	A: ReadOnlySystem,
	<A as System>::Out: IntoIterator,
	<A::Out as IntoIterator>::Item: IntoIterator,
{
}

impl<A> Clone for FlattenSystem<A>
where
	A: Clone,
{
	fn clone(&self) -> Self {
		FlattenSystem::new(self.a.clone(), self.name.clone())
	}
}
