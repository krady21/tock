//! Preemptive Priority Scheduler for Tock
//!
//! This scheduler allows for boards to set the priority of processes at boot,
//! and runs the highest priority process available at any point in time.
//! Kernel tasks (bottom half interrupt handling / deferred call handling)
//! always take priority over userspace processes.
//! Process priority is defined by the order the process appears in the PROCESSES
//! array. Notably, there is no need to enforce timeslices, as it is impossible
//! for a process running to not be the highest priority process at any point
//! without the process being descheduled.

use crate::capabilities;
use crate::common::dynamic_deferred_call::DynamicDeferredCall;
use crate::ipc;
use crate::platform::{Chip, Platform};
use crate::process;
use crate::sched;
use crate::sched::{Kernel, Scheduler};

/// Preemptive Priority Scheduler
pub struct PrioritySched {
    kernel: &'static Kernel,
}

impl PrioritySched {
    /// How long a process can run before being pre-empted
    pub const fn new(kernel: &'static Kernel) -> Self {
        Self { kernel }
    }
}

impl Scheduler for PrioritySched {
    // /// Main loop.
    // fn kernel_loop<P: Platform, C: Chip>(
    //     &self,
    //     platform: &P,
    //     chip: &C,
    //     ipc: Option<&ipc::IPC>,
    //     _capability: &dyn capabilities::MainLoopCapability,
    // ) -> ! {
    //     self.kernel.kernel_loop(platform, chip, ipc, || unsafe {
    //         for p in self.kernel.processes.iter() {
    //             p.map(|process| {
    //                 self.kernel
    //                     .do_process(platform, chip, &(), process, ipc, None, true)
    //             });
    //             if chip.has_pending_interrupts()
    //                 || DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
    //             {
    //                 break;
    //             }
    //         }
    //     })
    //     // loop {
    //     //     unsafe {
    //     //         chip.service_pending_interrupts();
    //     //         DynamicDeferredCall::call_global_instance_while(|| !chip.has_pending_interrupts());

    //     //         loop {
    //     //             if chip.has_pending_interrupts()
    //     //                 || DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
    //     //                 || self.kernel.processes_blocked()
    //     //             {
    //     //                 break;
    //     //             }

    //     //         }

    //     //         chip.atomic(|| {
    //     //             if !chip.has_pending_interrupts()
    //     //                 && !DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
    //     //                 && self.kernel.processes_blocked()
    //     //             {
    //     //                 chip.sleep();
    //     //             }
    //     //         });
    //     //     };
    //     // }
    // }

    fn next(&self) -> (Option<&'static dyn process::ProcessType>, u32) {
        (*self.kernel.processes.iter().nth(0).unwrap_or(&None), 10000)
    }

    fn result(&self, result: sched::StoppedExecutingReason) {}
}
