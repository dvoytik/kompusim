use std::{
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
    instr_cache: Option<Vec<u32>>,
    instr_cache_len: usize,
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

enum SimCommand {
    //Reset,
    //Init,
    /// LoadImage(load_address, image addres, breakpoint)
    NoCmd,
    Continue,
    Step,
    Stop,
    LoadImage((u64, &'static [u8], u64)),
    /// Disasm(starting_address, number_of_instruction)
    Disasm(u64, usize),
}

#[derive(Clone)]
enum SimEvent {
    /// SimState, registers, number of executed instructions
    StateChanged(SimState, Box<RV64IURegs>, u64),
    Instructions(Vec<u32>),
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
            let addr = 0x0000000080000000; // TODO: remove
            let ram_sz = 4 * 1024; // TODO: remove
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
            cpu0.regs.pc = addr;

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
                            println!("no commands");
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
                            if let ExecEvent::Breakpoint(_) = cpu0.exec_continue(102400) {
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
                        if let ExecEvent::Breakpoint(_) = cpu0.exec_continue(102400) {
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
                        cpu0.bus.load_image(load_addr, image).unwrap();
                        cpu0.add_breakpoint(breakpoint);
                        sim_state = SimState::Stopped;
                        send_event(SimEvent::StateChanged(
                            sim_state,
                            Box::new(cpu0.get_regs().clone()),
                            cpu0.get_num_exec_instr(),
                        ));
                        println!("Simulator: image loaded at 0x{:x}", load_addr);
                    }
                    SimCommand::Disasm(addr, n_instr) => {
                        let instructions = cpu0.get_n_instr(addr, n_instr);
                        send_event(SimEvent::Instructions(instructions));
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
            instr_cache_len: 0,
            event_queue: event_recv,
        }
    }

    pub fn stop(&mut self) {
        if self.sim_thread.is_some() {
            if let Err(err) = self.cmd_channel.send(SimCommand::Stop) {
                println!("Simulator: failed to send command: {}", err);
            }
            self.sim_thread.take().unwrap().join().unwrap();
        }
    }

    fn send_cmd(&self, cmd: SimCommand) {
        if let Err(e) = self.cmd_channel.send(cmd) {
            println!("FAILED to send command. Error: {}", e)
        }
    }

    pub fn load_image(&mut self, addr: u64, image: &'static [u8], breakpoint: u64) {
        self.send_cmd(SimCommand::LoadImage((addr, image, breakpoint)));
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
                self.instr_cache.replace(instructions);
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
            || pc >= self.instr_cache_start + self.instr_cache_len as u64 * 4
        {
            // update cache if needed
            let _ = self.get_instructions(pc, 4);
        }
        let offset = (pc - self.instr_cache_start) / 4;
        self.instr_cache.as_ref().unwrap()[offset as usize]
    }

    /// Returns (instructions_array, start_address)
    pub fn get_instructions(&mut self, start_addr: u64, num_instr: usize) -> (&Vec<u32>, u64) {
        if self.instr_cache.is_none()
            || start_addr < self.instr_cache_start
            || start_addr + num_instr as u64 * 4
                > self.instr_cache_start + self.instr_cache_len as u64 * 4
        {
            println!("Updating instruction cache"); // keep it for debuggin unnecessary cache updates
            self.send_cmd(SimCommand::Disasm(start_addr, num_instr));
            self.wait_for_event(SimEvent::Instructions(Vec::default()));
            self.instr_cache_start = start_addr;
            self.instr_cache_len = num_instr;
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
