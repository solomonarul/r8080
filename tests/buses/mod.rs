mod cpm_bus;
mod echo_bus;

#[allow(dead_code)]
pub type EchoBus = echo_bus::EchoBus;
pub type TestCPMBus = cpm_bus::TestCPMBus;
