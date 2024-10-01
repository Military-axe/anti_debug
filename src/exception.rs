use crate::util::BeingDebug;
use rand::{rngs::ThreadRng, Rng};

pub type HandlerFunc = Box<dyn Fn() -> bool>;
pub type AntiDebugHandlers = Vec<Box<dyn Fn() -> bool>>;

pub struct Exception {
    pub handlers: AntiDebugHandlers,
}

impl Exception {
    pub fn register_handler<T: BeingDebug>(&mut self, obj: &'static T) {
        // 将对象的方法转换为函数指针并添加到 handlers 中
        self.handlers.push(Box::new(move || obj.is_being_debug()));
    }

    pub fn rand_handlers(&self) -> Option<&HandlerFunc> {
        if self.handlers.is_empty() {
            return None;
        }

        let mut rng: ThreadRng = rand::thread_rng();
        let index = rng.gen_range(0..self.handlers.len());
        Some(&self.handlers[index])
    }
}
