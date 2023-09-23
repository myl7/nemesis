fn main() {
    tonic_build::configure()
        .compile(&["proto/eems.proto", "proto/user.proto"], &["proto"])
        .unwrap();
}
