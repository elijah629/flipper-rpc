// This file is @generated by prost-build.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetRequest {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetResponse {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
