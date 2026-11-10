fn main() {
    // Showcase interaction and geometry checks inspect elements by component
    // identity. This consumer explicitly opts in; the kit build helper keeps
    // Slint's metadata default for ordinary applications.
    kit_build::compile_with_config(
        "ui/showcase.slint",
        kit_build::config().with_debug_info(true),
    )
    .expect("compile the public kit consumer");
}
