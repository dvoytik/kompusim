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
    rv64i_disasm::disasm,
    uart::Uart,
};

type DisasmInstructionLine = (Option<&'static str>, String, String, String);

pub struct Simulator {
    sim_thread: Option<thread::JoinHandle<()>>,
    cmd_channel: Sender<SimCommand>,
    /// UART TX receive queue
    uart_tx_recv: Receiver<u8>,
    /// lock-less mirrored state of the simulator
    sim_state: SimState,
    regs: RV64IURegs,
    /// cached mirror of last received instructions; must be always updated with SimCommand::Disasm
    /// TODO: do we need to cache?
    instructions: Vec<u32>,
    event_queue: Receiver<SimEvent>,
    /// Cached disassembler listing
    disasm_listing: Option<Vec<DisasmInstructionLine>>,
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
    Stop,
    LoadImage((u64, &'static [u8], u64)),
    /// Disasm(starting_address, number_of_instruction)
    Disasm(u64, usize),
}

#[derive(Clone)]
enum SimEvent {
    StateChanged(SimState, RV64IURegs),
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
            send_event(SimEvent::StateChanged(sim_state, cpu0.get_regs().clone()));
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
                                    cpu0.get_regs().clone(),
                                ));
                            }
                        }
                    }
                    SimCommand::Continue => {
                        sim_state = SimState::Running;
                        if let ExecEvent::Breakpoint(_) = cpu0.exec_continue(102400) {
                            sim_state = SimState::StoppedBreakpoint;
                            send_event(SimEvent::StateChanged(sim_state, cpu0.get_regs().clone()));
                        }
                    }
                    // SimCommand::Reset => {
                    //     println!("Simulator: reset command")
                    // }
                    // SimCommand::Init => {}
                    SimCommand::LoadImage((load_addr, image, breakpoint)) => {
                        cpu0.bus.load_image(load_addr, image).unwrap();
                        cpu0.add_breakpoint(breakpoint);
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
            regs: RV64IURegs::default(),
            instructions: Vec::default(),
            event_queue: event_recv,
            disasm_listing: None,
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
        match self.cmd_channel.send(cmd) {
            Err(e) => println!("FAILED to send command. Error: {}", e),
            Ok(_) => {}
        }
    }

    pub fn load_image(&mut self, addr: u64, image: &'static [u8], breakpoint: u64) {
        self.send_cmd(SimCommand::LoadImage((addr, image, breakpoint)));
    }

    // continue is a Rust keyword, so use carry_on()
    pub fn carry_on(&self) {
        self.send_cmd(SimCommand::Continue);
    }

    fn process_event(&mut self, event: SimEvent) {
        match event {
            SimEvent::StateChanged(new_state, new_regs) => {
                self.sim_state = new_state;
                self.regs = new_regs;
                // clear disassembler cache
                self.disasm_listing.take();
            }
            SimEvent::Instructions(instructions) => {
                self.instructions = instructions;
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

    /// If sim is running, this will return stale registers from the last stop
    pub fn get_regs(&mut self) -> &RV64IURegs {
        self.drain_event_queue();
        &self.regs
    }

    pub fn disasm_at_pc(&mut self) -> &Vec<DisasmInstructionLine> {
        if self.disasm_listing.is_none() {
            let pc = self.regs.pc;
            self.send_cmd(SimCommand::Disasm(pc - 4, 32)); // TOOD: make parameters
            self.wait_for_event(SimEvent::Instructions(Vec::default()));
            //let start = (pc as i64 + pc_offset as i64) as u64;
            //tui::print_instr_listing(cpu0.get_n_instr(start, n_instr), start, pc);
            //
            let mut instr_addr = pc; // TODO: make parameter
            let mut instr_list: Vec<DisasmInstructionLine> = Vec::new();
            for instr in &self.instructions {
                let mark = if instr_addr == pc { Some("→") } else { None };
                let addr_hex = format!("0x{instr_addr}");
                let instr_hex = format!("0x{instr:08x}");
                let instr_mnemonic = disasm(*instr, instr_addr);
                instr_list.push((mark, addr_hex, instr_hex, instr_mnemonic));
                instr_addr += 4;
            }
            self.disasm_listing.replace(instr_list);
        }
        self.disasm_listing.as_ref().unwrap()
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
        if new_bytes.len() > 0 {
            Some(new_bytes)
        } else {
            None
        }
    }
}
