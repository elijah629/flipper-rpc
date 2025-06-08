//! Generic transport traits

pub mod serial;

/// Encodes, Decodes, Transports, and Receives data types
pub trait Transport<Send, Recv = Send> {
    /// Error type
    type Err: std::error::Error;

    /// Send a value of type `Send` over the transport.
    fn send(&mut self, value: Send) -> Result<(), Self::Err>;

    /// Receive a value of type `Recv` from the transport.
    fn receive(&mut self) -> Result<Recv, Self::Err>;

    /// Send a value, then immediately wait for and return a response.
    fn send_and_receive(&mut self, value: Send) -> Result<Recv, Self::Err> {
        self.send(value)?;
        self.receive()
    }
}

/// Transport with _raw suffixes
pub trait TransportRaw<Send, Recv = Send> {
    /// Error type
    type Err: std::error::Error;

    /// Send a value of type `Send` over the transport.
    fn send_raw(&mut self, value: Send) -> Result<(), Self::Err>;

    /// Receive a value of type `Recv` from the transport.
    fn receive_raw(&mut self) -> Result<Recv, Self::Err>;

    /// Send a value, then immediately wait for and return a response.
    fn send_and_receive_raw(&mut self, value: Send) -> Result<Recv, Self::Err> {
        self.send_raw(value)?;
        self.receive_raw()
    }
}
