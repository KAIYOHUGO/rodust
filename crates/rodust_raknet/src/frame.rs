use std::collections::HashMap;

use crate::{Fragment, Frame};

/// A buffer which collect fragment
#[derive(Debug, Clone, Default)]
pub struct Archaeologist {
    buffers: HashMap<u16, Buffer>,
}

impl Archaeologist {
    pub fn new() {
        Default::default()
    }

    pub fn collect(&mut self, frame: &Frame) -> Option<Vec<u8>> {
        // thanks harudagondi#1480@DC 👇
        let fragment = frame.fragment.as_ref()?;

        let state = if let Some(buffer) = self.buffers.get_mut(&fragment.compound_id) {
            buffer.read(&fragment, frame.body)
        } else {
            let mut buffer = Buffer::new(fragment.compound_size);
            let state = buffer.read(&fragment, frame.body);
            self.buffers.insert(fragment.compound_id, buffer);
            state
        };
        match state {
            BufferState::Ready => Some(
                self.buffers
                    .remove(&fragment.compound_id)
                    .expect("should not fail")
                    .buf,
            ),
            BufferState::Incomplete => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Buffer {
    counter: u32,
    size: u32,
    buf: Vec<u8>,
}
enum BufferState {
    Ready,
    Incomplete,
}
impl Buffer {
    fn new(size: u32) -> Self {
        Self {
            counter: 0,
            size,
            buf: vec![],
        }
    }

    fn read(&mut self, fragment: &Fragment, bytes: &[u8]) -> BufferState {
        if self.counter != fragment.index {
            return BufferState::Incomplete;
        }
        self.counter += 1;
        self.buf.extend_from_slice(bytes);
        if self.counter == self.size {
            BufferState::Ready
        } else {
            BufferState::Incomplete
        }
    }
}
