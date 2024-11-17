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
	message = "`{Self}` can not combine systems `{A}` and `{B}`",
	label = "invalid system combination",
	note = "the inputs and outputs of `{A}` and `{B}` are not compatible with this combiner"
)]
pub trait Do<A: System, B: System> {
	type In;

	fn combine(
		input: Self::In,
		a: impl FnOnce(A::In) -> A::Out,
		b: impl FnMut(B::In) -> B::Out,
	) -> A::Out;
}

pub struct DoerSystem<Func, A, B> {
	_marker: PhantomData<fn() -> Func>,
	a: A,
	b: B,
	name: Cow<'static, str>,
	component_access: Access<ComponentId>,
	archetype_component_access: Access<ArchetypeComponentId>,
}

impl<Func, A, B> DoerSystem<Func, A, B> {
	pub const fn new(a: A, b: B, name: Cow<'static, str>) -> Self {
		Self {
			_marker: PhantomData,
			a,
			b,
			name,
			component_access: Access::new(),
			archetype_component_access: Access::new(),
		}
	}
}

impl<A, B, Func> System for DoerSystem<Func, A, B>
where
	Func: Do<A, B> + 'static,
	A: System,
	B: System,
{
	type In = Func::In;
	type Out = A::Out;

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
			// SAFETY: See the comment above.
			|input| unsafe { self.b.run_unsafe(input, world) },
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
			#[allow(clippy::undocumented_unsafe_blocks)]
			|input| self.b.run(input, unsafe { world.deref_mut() }),
		)
	}

	fn apply_deferred(&mut self, world: &mut World) {
		self.a.apply_deferred(world);
		self.b.apply_deferred(world);
	}

	#[inline]
	fn queue_deferred(&mut self, mut world: bevy::ecs::world::DeferredWorld) {
		self.a.queue_deferred(world.reborrow());
		self.b.queue_deferred(world);
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

unsafe impl<A, B, Func> ReadOnlySystem for DoerSystem<Func, A, B>
where
	Func: Do<A, B> + 'static,
	A: ReadOnlySystem,
	B: ReadOnlySystem,
{
}

impl<Func, A, B> Clone for DoerSystem<Func, A, B>
where
	A: Clone,
	B: Clone,
{
	fn clone(&self) -> Self {
		DoerSystem::new(self.a.clone(), self.b.clone(), self.name.clone())
	}
}

pub type DoSystem<SystemA, SystemB> = DoerSystem<D, SystemA, SystemB>;

#[doc(hidden)]
pub struct D;

impl<A, B, I, II> Do<A, B> for D
where
	A: System<Out = I>,
	B: System<In = II, Out = ()>,
	I: IntoIterator<Item = II> + FromIterator<II>,
	II: Clone,
{
	type In = A::In;

	fn combine(
		input: Self::In,
		a: impl FnOnce(A::In) -> A::Out,
		mut b: impl FnMut(B::In) -> B::Out,
	) -> A::Out {
		let value = a(input);
		value.into_iter().inspect(|x| b(x.clone())).collect()
	}
}

pub trait DoSystemTrait<In, Out, Marker>: IntoSystem<In, Out, Marker>
where
	Out: IntoIterator,
{
	// This should actually be called iter_inspect
	fn iter_do<B, MarkerB>(self, system: B) -> DoSystem<Self::System, B::System>
	where
		B: IntoSystem<Out::Item, (), MarkerB>,
	{
		let system_a = IntoSystem::into_system(self);
		let system_b = IntoSystem::into_system(system);
		let name = format!("Do({}, {})", system_a.name(), system_b.name());
		DoSystem::new(system_a, system_b, Cow::Owned(name))
	}
}
impl<T, In, Out, Marker> DoSystemTrait<In, Out, Marker> for T
where
	T: IntoSystem<In, Out, Marker>,
	Out: IntoIterator,
{
}
