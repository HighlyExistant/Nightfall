use std::{any::{Any, TypeId}, cell::Ref, marker::PhantomData};

use crate::world::World;

use super::{FunctionTask, IntoTask, Task, TaskParam};
macro_rules! impl_tuple {
    ($($val:ident),*) => {
        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<$($val: TaskParam),*> TaskParam for ($($val,)*) {
            type State = ($($val::State,)*);
            type Item<'new> = ($($val::Item<'new>,)*);
            fn init_state(
                    resources: &World
                ) -> Self::State {
                ($($val::init_state(resources),)*)
            }
            fn retrieve<'world, 'state>(
                    state: &'state mut Self::State,
                    resources: &'world World
                ) -> Self::Item<'world> {
                    let ($($val,)*) = state;
                    (
                        $($val::retrieve($val, resources),)*
                    )
            }
        }
        impl<F: Send + Sync + 'static, $($val: TaskParam + Send + Sync + 'static),*> IntoTask<($($val,)*)> for F 
        where
            for<'a, 'b> &'a mut F: 
                FnMut($($val),*) + 
                FnMut($(<$val as TaskParam>::Item<'b>),*)
        {
            type Task = FunctionTask<($($val,)*), Self>;

            fn into_task(self) -> Self::Task {
                FunctionTask {
                    f: self,
                    inputs: None,
                }
            }
        }
        
        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<F: Send + Sync, $($val: TaskParam + Send + Sync),*> Task for FunctionTask<($($val,)*), F> 
        where
        for<'a, 'b> &'a mut F:
            FnMut($($val),*) +
            FnMut($(<$val as TaskParam>::Item<'b>),*)
        {
            fn execute(&mut self, resources: &World) {
                fn call_inner<$($val),*>(
                    mut f: impl FnMut($($val),*),
                    $($val: $val),*
                ) {
                    f($($val),*)
                }
                if let Some(($($val,)*)) = &mut self.inputs {
                    $(let $val = $val::retrieve($val, resources);)*
                    call_inner(&mut self.f, $($val),*)
                }
            }
            fn initialize(&mut self, resources: &World) {
                self.inputs = Some(
                    ($($val::init_state(resources),)*)
                );
            }
        }
    };
}
impl_tuple!(T1);
impl_tuple!(T1, T2);
impl_tuple!(T1, T2, T3);
impl_tuple!(T1, T2, T3, T4);
impl_tuple!(T1, T2, T3, T4, T5);
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

pub struct Res<'a, T: 'static> {
    value: Ref<'a, Box<dyn Any>>,
    _marker: PhantomData<&'a T>,
}
unsafe impl<'res, T: 'static> Send for Res<'res, T> {}
unsafe impl<'res, T: 'static> Sync for Res<'res, T> {}

impl<'res, T: 'static> TaskParam for Res<'res, T> {
    type State = usize;
    type Item<'new> = Res<'new, T>;
    fn init_state(
            resources: &World
        ) -> Self::State {
        0
    }
    fn retrieve<'world, 'state>(
            state: &'state mut Self::State,
            resources: &'world World
        ) -> Self::Item<'world> {
        Res { value: resources.get_resource(&TypeId::of::<T>()).unwrap(), _marker: PhantomData }
    }
}