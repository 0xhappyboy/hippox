pub mod Config {
    // Service port configuration
    pub const TCP_PORT: u16 = 8080;
    pub const HTTP_PORT: u16 = 8081;
    pub const WEBSOCKET_PORT: u16 = 8082;
    pub const GRPC_PORT: u16 = 50051;

    // Service address configuration
    pub const TCP_ADDR: &str = "127.0.0.1";
    pub const HTTP_ADDR: &str = "127.0.0.1";
    pub const WEBSOCKET_ADDR: &str = "127.0.0.1";
    pub const GRPC_ADDR: &str = "127.0.0.1";

    // Full address strings
    pub fn tcp_address() -> String {
        format!("{}:{}", TCP_ADDR, TCP_PORT)
    }

    pub fn http_address() -> String {
        format!("{}:{}", HTTP_ADDR, HTTP_PORT)
    }

    pub fn websocket_address() -> String {
        format!("{}:{}", WEBSOCKET_ADDR, WEBSOCKET_PORT)
    }

    pub fn grpc_address() -> String {
        format!("{}:{}", GRPC_ADDR, GRPC_PORT)
    }

    // Skills directory configuration
    pub const SKILLS_DIR: &str = "./skills";
}
