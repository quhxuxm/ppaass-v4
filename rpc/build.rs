fn main() {
    tonic_build::compile_protos("proto/rpc.proto").expect("Failed to compile proto")
}
