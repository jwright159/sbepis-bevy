use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::marker::PhantomData;

use bevy::ecs::archetype::ArchetypeComponentId;
use bevy::ecs::component::{ComponentId, Tick};
use bevy::ecs::query::Access;
use bevy::ecs::schedule::InternedSystemSet;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy::ptr::UnsafeCellDeref;

#[diagnostic::on_unimplemented(
	message = "`{Self}` can not combine systems `{A}` and `B`",
	label = "invalid system combination",
	note = "the inputs and outputs of `{A}` and `B` are not compatible with this combiner"
)]
pub trait Filter<A: System> {
	type In;
	type Out;

	fn combine(input: Self::In, a: impl FnOnce(A::In) -> A::Out) -> Self::Out;
}

pub struct FilterSomeSystem<Func, A> {
	_marker: PhantomData<fn() -> Func>,
	a: A,
	name: Cow<'static, str>,
	component_access: Access<ComponentId>,
	archetype_component_access: Access<ArchetypeComponentId>,
}

impl<Func, A> FilterSomeSystem<Func, A> {
	pub const fn new(a: A, name: Cow<'static, str>) -> Self {
		Self {
			_marker: PhantomData,
			a,
			name,
			component_access: Access::new(),
			archetype_component_access: Access::new(),
		}
	}
}

impl<A, Func> System for FilterSomeSystem<Func, A>
where
	Func: Filter<A> + 'static,
	A: System,
{
	type In = Func::In;
	type Out = Func::Out;

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

	unsafe fn run_unsafe(&mut self, input: Self::In, world: UnsafeWorldCell) -> Self::Out {
		Func::combine(
			input,
			// SAFETY: The world accesses for both underlying systems have been registered,
			// so the caller will guarantee that no other systems will conflict with `a` or `b`.
			// Since these closures are `!Send + !Sync + !'static`, they can never be called
			// in parallel, so their world accesses will not conflict with each other.
			// Additionally, `update_archetype_component_access` has been called,
			// which forwards to the implementations for `self.a` and `self.b`.
			|input| unsafe { self.a.run_unsafe(input, world) },
		)
	}

	fn run<'w>(&mut self, input: Self::In, world: &'w mut World) -> Self::Out {
		// SAFETY: Converting `&mut T` -> `&UnsafeCell<T>`
		// is explicitly allowed in the docs for `UnsafeCell`.
		let world: &'w UnsafeCell<World> = unsafe { std::mem::transmute(world) };
		Func::combine(
			input,
			// SAFETY: Since these closures are `!Send + !Sync + !'static`, they can never
			// be called in parallel. Since mutable access to `world` only exists within
			// the scope of either closure, we can be sure they will never alias one another.
			|input| self.a.run(input, unsafe { world.deref_mut() }),
		)
	}

	fn apply_deferred(&mut self, world: &mut World) {
		self.a.apply_deferred(world);
	}

	#[inline]
	fn queue_deferred(&mut self, world: bevy::ecs::world::DeferredWorld) {
		self.a.queue_deferred(world);
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

unsafe impl<A, Func> ReadOnlySystem for FilterSomeSystem<Func, A>
where
	Func: Filter<A> + 'static,
	A: ReadOnlySystem,
{
}

impl<Func, A> Clone for FilterSomeSystem<Func, A>
where
	A: Clone,
{
	fn clone(&self) -> Self {
		FilterSomeSystem::new(self.a.clone(), self.name.clone())
	}
}

pub type FSomeSystem<SystemA> = FilterSomeSystem<FSome, SystemA>;

#[doc(hidden)]
pub struct FSome;

impl<A, I, II> Filter<A> for FSome
where
	A: System<Out = I>,
	I: IntoIterator<Item = Option<II>>,
{
	type In = A::In;
	type Out = Vec<II>;

	fn combine(input: Self::In, a: impl FnOnce(A::In) -> A::Out) -> Self::Out {
		let value = a(input);
		value.into_iter().flatten().collect()
	}
}

pub trait FilterOkSystemTrait<In, Out, Marker>: IntoSystem<In, Out, Marker>
where
	Out: IntoIterator,
{
	// This should actually be called iter_flatten and constrained to IntoIterator instead of Option
	fn iter_filter_some(self) -> FSomeSystem<Self::System> {
		let system_a = IntoSystem::into_system(self);
		let name = format!("FSome({})", system_a.name());
		FSomeSystem::new(system_a, Cow::Owned(name))
	}
}
impl<T, In, Out, Marker> FilterOkSystemTrait<In, Out, Marker> for T
where
	T: IntoSystem<In, Out, Marker>,
	Out: IntoIterator,
{
}
