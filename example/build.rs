extern crate prost_build;
extern crate prost_simple_rpc_build;

fn main() {
    prost_build::Config::new()
        .service_generator(Box::new(prost_simple_rpc_build::ServiceGenerator::new()))
        .compile_protos(
            &[
                "src/schema/echo/service.proto",
                "src/schema/greeting/service.proto",
            ],
            &["src/schema"],
        )
        .unwrap();
}
