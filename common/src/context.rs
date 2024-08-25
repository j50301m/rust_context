use core::cell::RefCell;
use std::any::{Any, TypeId};
use std::hash::{BuildHasherDefault, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;

use std::collections::HashMap;

thread_local! {
    static CURRENT_CONTEXT: RefCell<Context> = RefCell::new(Context::new());
}

/// [`Context`]是一個管理線程上下文的結構體
#[derive(Default, Clone)]
pub struct Context {
    entries: HashMap<TypeId, Arc<dyn Any + Sync + Send>, BuildHasherDefault<IdHasher>>,
}

impl Context {
    pub fn new() -> Self {
        Context::default()
    }

    pub fn current() -> Self {
        Context::map_current(|cx| cx.clone())
    }

    pub fn map_current<T>(f: impl FnOnce(&Context) -> T) -> T {
        CURRENT_CONTEXT.with(|cx| f(&cx.borrow()))
    }

    pub fn current_with_value<T: 'static + Send + Sync>(value: T) -> Self {
        let mut new_context = Context::current();
        new_context
            .entries
            .insert(TypeId::of::<T>(), Arc::new(value));

        new_context
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.entries
            .get(&TypeId::of::<T>())
            .and_then(|rc| rc.downcast_ref())
    }

    pub fn with_value<T: 'static + Send + Sync>(&self, value: T) -> Self {
        let mut new_context = self.clone();
        new_context
            .entries
            .insert(TypeId::of::<T>(), Arc::new(value));

        new_context
    }

    pub fn attach(self) -> ContextGuard {
        let previous_cx = CURRENT_CONTEXT
            .try_with(|current| current.replace(self))
            .ok();

        ContextGuard {
            previous_cx,
            _marker: PhantomData,
        }
    }

    pub fn try_move_out<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        self.entries.remove(&TypeId::of::<T>()).and_then(|rc| {
            // Downcast Arc<dyn Any + Send + Sync> to Arc<T>
            let arc = rc.downcast::<T>().ok()?;
            // Unwrap Arc<T> to get the inner T
            Arc::try_unwrap(arc).ok()
        })
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("entries", &self.entries.len())
            .finish()
    }
}

#[allow(missing_debug_implementations)]
pub struct ContextGuard {
    previous_cx: Option<Context>,
    // ensure this type is !Send as it relies on thread locals
    _marker: PhantomData<*const ()>,
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        if let Some(previous_cx) = self.previous_cx.take() {
            let _ = CURRENT_CONTEXT.try_with(|current| current.replace(previous_cx));
        }
    }
}

/// With TypeIds as keys, there's no need to hash them. They are already hashes
/// themselves, coming from the compiler. The IdHasher holds the u64 of
/// the TypeId, and then returns it, instead of doing any bit fiddling.
#[derive(Clone, Default, Debug)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct ValueA(&'static str);
    #[derive(Debug, PartialEq)]
    struct ValueB(u64);

    #[test]
    fn nested_contexts() {
        #[derive(Debug, PartialEq)]
        struct ValueA(&'static str);
        #[derive(Debug, PartialEq)]
        struct ValueB(u64);

        // Set the context guard
        let _outer_guard = Context::new().with_value(ValueA("a")).attach();

        let current = Context::current();
        assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
        assert_eq!(current.get::<ValueB>(), None);

        {
            let _inner_guard = Context::current_with_value(ValueB(42)).attach();
            let current = Context::current();
            assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
            assert_eq!(current.get::<ValueB>(), Some(&ValueB(42)));

            Context::map_current(|cx| {
                assert_eq!(cx.get::<ValueA>(), Some(&ValueA("a")));
                assert_eq!(cx.get::<ValueB>(), Some(&ValueB(42)));
            });
        }

        // Restore the outer context
        let current = Context::current();
        assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
        assert_eq!(current.get::<ValueB>(), None);

        Context::map_current(|cx| {
            assert_eq!(cx.get::<ValueA>(), Some(&ValueA("a")));
            assert_eq!(cx.get::<ValueB>(), None);
        });
    }

    fn foo() {
        let _guard = Context::new()
            .with_value(ValueB(42))
            .with_value(ValueA("foo"))
            .attach();
        let current = Context::current();
        assert_eq!(current.get::<ValueA>(), Some(&ValueA("foo")));
        assert_eq!(current.get::<ValueB>(), Some(&ValueB(42)));
    }

    fn bar() {
        let _guard = Context::new().with_value(ValueA("bar")).attach();
        let current = Context::current();
        assert_eq!(current.get::<ValueA>(), Some(&ValueA("bar")));
        assert_eq!(current.get::<ValueB>(), None);
    }

    #[test]
    fn enter_function() {
        let _guard = Context::new().with_value(ValueA("a")).attach();

        let current = Context::current();
        assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
        assert_eq!(current.get::<ValueB>(), None);

        // Enter other function with different context, and check the context is restored
        foo();
        assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
        assert_eq!(current.get::<ValueB>(), None);

        // Enter other function with different context again, and check the context is restored
        bar();
        assert_eq!(current.get::<ValueA>(), Some(&ValueA("a")));
        assert_eq!(current.get::<ValueB>(), None);
    }
}
