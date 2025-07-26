fn main() {
    tonic_build::configure()
        .compile_protos(
            &[
                "proto/values.proto",
                "proto/session.proto",
                "proto/connection.proto",
                "proto/relay.proto",
                "proto/process.proto",
            ],
            &["proto"],
        )
        .expect("Fail to compile proto");
}
