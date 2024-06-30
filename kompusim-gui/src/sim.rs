/*
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError},
    thread,
    time::Duration,
};

use kompusim::{
    bus,
    device::Device,
    ram,
    rv64i_cpu::{ExecEvent, RV64ICpu, RV64IURegs},
    uart::Uart,
};

// TODO: setting
const EXE_INSTRUCTIONS_THEN_POLL: u64 = 102400;

pub const DEFAULT_MEM_SZ: u64 = 1024 * 1024;
pub const DEFAULT_START_ADDRESS: u64 = 0x8000_0000;

pub struct Simulator {
    sim_thread: Option<thread::JoinHandle<()>>,
    cmd_channel: Sender<SimCommand>,
    /// UART TX receive queue
    uart_tx_recv: Receiver<u8>,
    /// lock-less mirrored state of the simulator
    sim_state: SimState,
    event_queue: Receiver<SimEvent>,
    regs: Box<RV64IURegs>,
    /// lock-less mirrored number of executed instructions
    num_exec_instr: u64,
    /// cached mirror of last received instructions; must be always updated with SimCommand::Disasm
    /// TODO: move to general memory cache - no need to diffirentiate instruction cache from memory
    /// cache
    instr_cache: Option<Vec<u8>>,
    // instruction cache size in bytes
    instr_cache_sz: u64,
    instr_cache_start: u64,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SimState {
    /// The simulator transitions to Initialized state after first init phase
    Initializing,
    InitializedReady,
    StoppedBreakpoint,
    Stopped,
    Running,
}

#[derive(Debug, Clone)]
enum LoadImageType {
    File(PathBuf),
    StaticMem(&'static [u8]),
}

#[derive(Debug, Clone)]
enum SimCommand {
    //Reset,
    //Init,
    /// LoadImage(load_address, image addres, breakpoint)
    NoCmd,
    Continue,
    Step,
    Stop,
    LoadImage((u64, LoadImageType, u64)),
    /// Disasm(starting_address, number_of_bytes)
    Disasm(u64, u64),
    // Set RAM size
    SetRamSz(u64),
    // Add new breakpoint
    AddBreakpoint(u64),
}

#[derive(Clone)]
enum SimEvent {
    /// SimState, registers, number of executed instructions
    StateChanged(SimState, Box<RV64IURegs>, u64),
    Instructions(Option<Vec<u8>>),
}

impl Simulator {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx): (Sender<SimCommand>, Receiver<SimCommand>) = mpsc::channel();
        // TODO: move to the Event queue
        let (uart_tx_send, uart_tx_recv): (Sender<u8>, Receiver<u8>) = mpsc::channel();
        let (event_send, event_recv): (Sender<SimEvent>, Receiver<SimEvent>) = mpsc::channel();

        // Start the simulator thread
        let sim_thread_handler = thread::spawn(move || {
            let send_event = |e| {
                if let Err(err) = event_send.send(e) {
                    eprintln!("Simulator: failed to send event: {}", err);
                }
            };
            let addr = DEFAULT_START_ADDRESS;
            let ram_sz = DEFAULT_MEM_SZ;
            let ram = ram::Ram::new(addr, ram_sz);
            let mut bus = bus::Bus::new();
            bus.attach_ram(ram);

            let mut uart0 = Box::new(Uart::new("0".to_string()));
            uart0.register_out_callback(Box::new(move |b: u8| {
                if let Err(err) = uart_tx_send.send(b) {
                    println!("Simulator: failed to send command: {}", err);
                }
            }));
            bus.attach_device(Device::new(uart0, 0x1001_0000, 0x20));

            let mut cpu0 = RV64ICpu::new(bus);
            cpu0.pc_jump(addr);

            let mut sim_state = SimState::InitializedReady;
            send_event(SimEvent::StateChanged(
                sim_state,
                Box::new(cpu0.get_regs().clone()),
                0,
            ));
            loop {
                // In non-running state we block on empty command channel
                let recv_cmd = if sim_state != SimState::Running {
                    cmd_rx.recv().unwrap()
                } else {
                    // In SimState:Running we poll command channel
                    match cmd_rx.try_recv() {
                        Err(TryRecvError::Empty) => {
                            // TODO: state machine
                            println!(
                                "executed {EXE_INSTRUCTIONS_THEN_POLL} instructions. \
                                Polling new commands. Tip: set a breakpoint to stop simulator."
                            );
                            SimCommand::NoCmd
                        }
                        Err(TryRecvError::Disconnected) => {
                            eprintln!("ERROR: disconnected from the cmd channel");
                            break;
                        }
                        Ok(cmd) => cmd,
                    }
                };
                match recv_cmd {
                    SimCommand::NoCmd => {
                        if sim_state == SimState::Running {
                            // TODO: move to settings
                            if let ExecEvent::Breakpoint(_) =
                                cpu0.exec_continue(EXE_INSTRUCTIONS_THEN_POLL)
                            {
                                sim_state = SimState::StoppedBreakpoint;
                                send_event(SimEvent::StateChanged(
                                    sim_state,
                                    Box::new(cpu0.get_regs().clone()),
                                    cpu0.get_num_exec_instr(),
                                ));
                            }
                        }
                    }
                    SimCommand::Continue => {
                        sim_state = SimState::Running;
                        if let ExecEvent::Breakpoint(_) =
                            cpu0.exec_continue(EXE_INSTRUCTIONS_THEN_POLL)
                        {
                            sim_state = SimState::StoppedBreakpoint;
                            send_event(SimEvent::StateChanged(
                                sim_state,
                                Box::new(cpu0.get_regs().clone()),
                                cpu0.get_num_exec_instr(),
                            ));
                        }
                    }
                    SimCommand::Step => {
                        let _ = cpu0.exec_continue(1);
                        sim_state = SimState::Stopped;
                        send_event(SimEvent::StateChanged(
                            sim_state,
                            Box::new(cpu0.get_regs().clone()),
                            cpu0.get_num_exec_instr(),
                        ));
                    }
                    // SimCommand::Reset => {
                    //     println!("Simulator: reset command")
                    // }
                    // SimCommand::Init => {}
                    SimCommand::LoadImage((load_addr, image, breakpoint)) => {
                        match image {
                            LoadImageType::File(file_path) => {
                                cpu0.bus.load_file(load_addr, &file_path).unwrap();
                            }
                            LoadImageType::StaticMem(mem_buf) => {
                                cpu0.bus.load_image(load_addr, mem_buf).unwrap();
                            }
                        }
                        cpu0.add_breakpoint(breakpoint);
                        sim_state = SimState::Stopped;
                        send_event(SimEvent::StateChanged(
                            sim_state,
                            Box::new(cpu0.get_regs().clone()),
                            cpu0.get_num_exec_instr(),
                        ));
                        println!("Simulator: image loaded at 0x{:x}", load_addr);
                    }
                    SimCommand::Disasm(addr, n_bytes) => {
                        let instructions = cpu0
                            .get_ram(addr, n_bytes)
                            .map(|mem_area| mem_area.to_owned());
                        send_event(SimEvent::Instructions(instructions));
                    }
                    SimCommand::SetRamSz(ram_sz) => {
                        cpu0.set_ram_sz(ram_sz);
                    }
                    SimCommand::AddBreakpoint(breakpoint) => {
                        cpu0.add_breakpoint(breakpoint);
                    }
                    SimCommand::Stop => break,
                }
                //thread::sleep(time::Duration::from_secs(1));
                // TODO: receive commands from the gui main thread
            }
            println!("Simulator: exiting the simulator thread");
        });
        Simulator {
            sim_thread: Some(sim_thread_handler),
            cmd_channel: cmd_tx,
            uart_tx_recv,
            sim_state: SimState::Initializing,
            regs: Box::<kompusim::rv64i_cpu::RV64IURegs>::default(),
            num_exec_instr: 0,
            instr_cache: None,
            instr_cache_start: 0,
            instr_cache_sz: 0,
            event_queue: event_recv,
        }
    }

    pub fn set_ram_sz(&mut self, ram_sz: u64) {
        self.send_cmd(SimCommand::SetRamSz(ram_sz));
    }

    pub fn add_breakpoint(&mut self, breakpoint: u64) {
        self.send_cmd(SimCommand::AddBreakpoint(breakpoint))
    }

    pub fn stop(&mut self) {
        if self.sim_thread.is_some() {
            self.send_cmd(SimCommand::Stop);
            self.sim_thread.take().unwrap().join().unwrap();
        }
    }

    fn send_cmd(&self, cmd: SimCommand) {
        // SimCommand is cheap to clone
        if let Err(e) = self.cmd_channel.send(cmd.clone()) {
            println!("Simulator: FAILED to send command {cmd:?}. Error: {e}")
        }
    }

    pub fn load_image(&mut self, addr: u64, image: &'static [u8], breakpoint: u64) {
        self.send_cmd(SimCommand::LoadImage((
            addr,
            LoadImageType::StaticMem(image),
            breakpoint,
        )));
        // clear disassembler cache - force loading instructions
        self.instr_cache.take();
    }

    pub fn load_bin_file(&mut self, addr: u64, image: PathBuf) {
        self.send_cmd(SimCommand::LoadImage((
            addr,
            LoadImageType::File(image),
            0x0,
        )));
        // clear disassembler cache - force loading instructions
        self.instr_cache.take();
    }

    // continue is a Rust keyword, so use carry_on()
    pub fn carry_on(&self) {
        self.send_cmd(SimCommand::Continue);
    }

    pub fn step(&self) {
        self.send_cmd(SimCommand::Step);
    }

    fn process_event(&mut self, event: SimEvent) {
        match event {
            SimEvent::StateChanged(new_state, new_regs, num_exec_instr) => {
                self.sim_state = new_state;
                self.regs = new_regs;
                self.num_exec_instr = num_exec_instr;
                // clear instruction cache
                // TODO: optimize - use memory watchpoints
                self.instr_cache.take();
            }
            SimEvent::Instructions(instructions) => {
                self.instr_cache = instructions;
            }
        }
    }

    fn drain_event_queue(&mut self) {
        let events: Vec<SimEvent> = self.event_queue.try_iter().collect();
        for event in events {
            self.process_event(event);
        }
    }

    fn wait_for_event(&mut self, expected_event: SimEvent) {
        loop {
            let event = match self.event_queue.recv_timeout(Duration::from_millis(5000)) {
                Ok(e) => e,
                // TODO: propagate the error up
                Err(RecvTimeoutError::Timeout) => {
                    eprintln!("ERROR: event queue failed timed out");
                    return;
                }
                // TODO: propagate the error up
                Err(RecvTimeoutError::Disconnected) => {
                    eprintln!("ERROR: event queue disconnected");
                    return;
                }
            };
            if std::mem::discriminant(&event) == std::mem::discriminant(&expected_event) {
                self.process_event(event);
                break;
            }
            self.process_event(event);
        }
    }

    pub fn get_state(&mut self) -> SimState {
        self.drain_event_queue(); // will update self.sim_state
        self.sim_state
    }

    pub fn get_num_exec_instr(&mut self) -> u64 {
        self.drain_event_queue(); // will update self.num_exec_instr
        self.num_exec_instr
    }

    /// If sim is running, this will return stale registers from the last stop
    pub fn get_regs(&mut self) -> &RV64IURegs {
        self.drain_event_queue();
        &self.regs
    }

    pub fn get_cur_instr(&mut self) -> u32 {
        let pc = self.regs.pc;
        if self.instr_cache.is_none()
            || pc < self.instr_cache_start
            || pc >= self.instr_cache_start + self.instr_cache_sz - 2
        {
            // update cache if needed
            let _ = self.get_instructions(pc, 4);
        }
        let offset = (pc - self.instr_cache_start) as usize;
        let instr0 = self.instr_cache.as_ref().unwrap()[offset];
        let instr1 = self.instr_cache.as_ref().unwrap()[offset + 1];
        let instr2 = self.instr_cache.as_ref().unwrap()[offset + 2];
        let instr3 = self.instr_cache.as_ref().unwrap()[offset + 3];
        (instr0 as u32) | (instr1 as u32) << 8 | (instr2 as u32) << 16 | (instr3 as u32) << 24
    }

    /// Returns memory area state (bytes_array, start_address)
    // pub fn get_mem_area(&mut self, start_addr: u64, size: usize) -> (&Vec<u32>, u64) {
    // }

    // TODO: remove and replace with get_mem_area()
    /// Returns (bytes_array, start_address)
    pub fn get_instructions(&mut self, start_addr: u64, size: u64) -> (&Vec<u8>, u64) {
        if self.instr_cache.is_none()
            || start_addr < self.instr_cache_start
            || start_addr + size > self.instr_cache_start + self.instr_cache_sz
        {
            println!("sim: Updating instruction cache"); // keep it for debuggin unnecessary cache updates
            self.send_cmd(SimCommand::Disasm(start_addr, size));
            self.wait_for_event(SimEvent::Instructions(Some(Vec::default())));
            self.instr_cache_start = start_addr;
            self.instr_cache_sz = size;
        }
        (self.instr_cache.as_ref().unwrap(), self.instr_cache_start)
    }

    pub fn console_recv(&self) -> Option<String> {
        // TODO: pass &String and push to it instead of allocating every time
        let mut new_bytes = String::new();
        // TODO: use .try_iter()
        loop {
            match self.uart_tx_recv.try_recv() {
                Ok(byte) => new_bytes.push(byte as char),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    eprintln!(
                        "Simulator: FATAL ERROR: got Disconnected on UART TX receive attempt"
                    );
                    break;
                }
            }
        }
        if !new_bytes.is_empty() {
            Some(new_bytes)
        } else {
            None
        }
    }
}
*/
