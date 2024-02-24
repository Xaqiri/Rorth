pub mod stack {
    use std::collections::HashSet;

    pub enum CurStack {
        Num,
        String,
    }

    pub fn new() -> Stack {
        Stack {
            stack: vec![],
            string_stack: vec![],
            string_heap: HashSet::new(),
            top_type: CurStack::Num,
            if_stack: vec![],
        }
    }

    pub struct Stack {
        pub stack: Vec<f32>,
        pub string_stack: Vec<String>,
        pub string_heap: HashSet<String>,
        pub top_type: CurStack,
        pub if_stack: Vec<usize>,
    }

    impl Stack {
        pub fn push(&mut self, value: f32) {
            self.stack.push(value);
        }
        pub fn push_str(&mut self, value: String) {
            self.string_stack.push(value);
        }
        pub fn push_if(&mut self) {
            self.if_stack.push(0);
        }
        pub fn pop(&mut self) -> Option<f32> {
            self.stack.pop()
        }
        pub fn pop_str(&mut self) -> Option<String> {
            self.string_stack.pop()
        }
        pub fn last(&self) -> Option<&f32> {
            self.stack.last()
        }
        pub fn len(&self) -> usize {
            self.stack.len()
        }
    }
}
