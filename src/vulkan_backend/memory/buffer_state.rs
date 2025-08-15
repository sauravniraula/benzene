pub enum VBufferState {
    UNMAPPED,
    MAPPED(*mut u8),
}
