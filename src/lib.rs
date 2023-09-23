pub mod crypto;
pub mod db;
pub mod eems;
pub mod user;

pub mod grpc {
    pub mod eems {
        tonic::include_proto!("eems");
    }
    pub mod user {
        tonic::include_proto!("user");
    }
}
