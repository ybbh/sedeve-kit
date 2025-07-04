use clap::Parser;
use scupt_util::init_logger::logger_setup;
use sedeve_kit::trace::trace_gen_portal;

fn main() {
    let args = trace_gen_portal::GenArgs::parse();
    logger_setup();
    trace_gen_portal::portal(args);
}
