use std::{any::{Any, TypeId}, cell::{Ref, RefCell}, collections::HashMap, marker::PhantomData};

use crate::world::World;

use super::{IntoTask, Task, TypeIdMap};
pub trait TaskParam {
    type State: Send + Sync + 'static;
    type Item<'new>: TaskParam<State = Self::State>;
    fn init_state(
        resources: &World
    ) -> Self::State;
    fn retrieve<'world, 'state>(
        state: &'state mut Self::State,
        resources: &'world World
    ) -> Self::Item<'world>;
}
// impl<F: Send + Sync + 'static, T1: TaskParam + Send + Sync + 'static> IntoTask<(T1,)> for F 
// where
//     for<'a, 'b> &'a mut F: 
//         FnMut(T1,) + 
//         FnMut(<T1 as TaskParam>::Item<'b>,)
// {
//     type Task = FunctionTask<(T1,), Self>;

//     fn into_task(self) -> Self::Task {
//         FunctionTask {
//             f: self,
//             inputs: None,
//         }
//     }
// }
// impl<T: TaskParam> TaskParam for (T,) {
//     type State = (T::State,);
//     type Item<'new> = (T::Item<'new>,);
//     fn init_state(
//             resources: &World
//         ) -> Self::State {
//         (T::init_state(resources),)
//     }
//     fn retrieve<'world, 'state>(
//             state: &'state mut Self::State,
//             resources: &'world World
//         ) -> Self::Item<'world> {
//             let (_0,) = state;
//             (T::retrieve(_0, resources),)
//     }
// }
pub struct FunctionTask<T: 'static + TaskParam, F: 'static> {
    pub(crate) f: F,
    pub(crate) inputs: Option<T::State>,
}
// impl<F: Send + Sync, T1: TaskParam + Send + Sync> Task for FunctionTask<(T1,), F> 
// where
// for<'a, 'b> &'a mut F:
//     FnMut(T1) +
//     FnMut(<T1 as TaskParam>::Item<'b>)
// {
//     fn execute(&mut self, resources: &World) {
//         fn call_inner<T1>(
//             mut f: impl FnMut(T1),
//             _0: T1,
//         ) {
//             f(_0)
//         }
//         if let Some((state,)) = &mut self.inputs {
//             let _0 = T1::retrieve(state, resources);
//             call_inner(&mut self.f, _0)
//         }
//     }
//     fn initialize(&mut self, resources: &World) {
//         self.inputs = Some((T1::init_state(resources),));
//     }
// }