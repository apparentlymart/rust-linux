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

    {
        let mem_data = mem.as_mut_slice();

        #[cfg(target_arch = "x86_64")]
        {
            // "UD2" explicitly-undefined instruction
            mem_data[0] = 0x0f;
            mem_data[1] = 0x0b;
        }

        #[cfg(target_arch = "aarch64")]
        {
            // "udf" explicitly-undefined instruction
            mem_data[0] = 0x01;
            mem_data[1] = 0x00;
            mem_data[2] = 0x00;
            mem_data[3] = 0x00;
        }
    }

    vm.set_guest_memory_region(0, KVM_MEM_READONLY, 0x1000, &mut mem)?;
    println!("registered the memory region with the VM");

    let cpu = vm.create_vcpu(0)?;
    println!("created a VCPU: {:?}", &cpu);
    let mut runner = cpu.to_runner()?;
    println!("created a VCPU runner: {:?}", &runner);
    runner.with_raw_run_state(|state| println!("run state: {:?}", &state));

    runner.modify_regs(|regs| {
        #[cfg(target_arch = "x86_64")]
        {
            regs.rip = 0x1000;
        }
        #[cfg(target_arch = "aarch64")]
        {
            regs.regs.pc = 0x1000;
        }
    })?;
    println!("set initial register values");

    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        runner.run_raw()?;
        runner.with_raw_run_state(|state| println!("run state after running: {:?}", &state));
        let regs = runner.get_regs()?;
        println!("registers after running: {:?}", &regs)
    }

    Ok(())
}
