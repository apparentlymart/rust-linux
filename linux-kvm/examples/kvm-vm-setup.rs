use linux_kvm::{ioctl::vm::KVM_MEM_READONLY, Kvm, MemoryRegion};

fn main() -> std::io::Result<()> {
    run().map_err(|e| e.into_std_io_error())
}

fn run() -> linux_io::result::Result<()> {
    let kvm = Kvm::open()?;
    println!("opened KVM subsystem: {:?}", &kvm);
    let version = kvm.get_api_version()?;
    if version != 12 {
        eprintln!("unsupported KVM API version {}", version);
        return Err(linux_io::result::Error::new(25));
    }
    let mut vm = kvm.create_vm()?;
    println!("created a VM: {:?}", &vm);

    let mut mem = MemoryRegion::new(0x1000)?;
    println!("created a memory region: {:?}", &mem);
    vm.set_guest_memory_region(0, KVM_MEM_READONLY, 0x1000, &mut mem)?;
    println!("registered the memory region with the VM");

    let cpu = vm.create_vcpu(0)?;
    println!("created a VCPU: {:?}", &cpu);
    let mut runner = cpu.to_runner()?;
    println!("created a VCPU runner: {:?}", &runner);
    runner.with_raw_run_state(|state| println!("run state: {:?}", &state));

    // We can't actually run the VM because we haven't put any code in there
    // to run and even if we did it would be architecture-specific, but the
    // above at least shows how to get everything allocated.

    Ok(())
}
