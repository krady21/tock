//! Round Robin Scheduler for Tock
//! This scheduler is specifically a Round Robin Scheduler with Interrupts.
//! See: https://www.eecs.umich.edu/courses/eecs461/lecture/SWArchitecture.pdf
//! for details.
//! When hardware interrupts occur while a userspace process is executing,
//! this scheduler executes the top half of the interrupt,
//! and then stops executing the userspace process immediately and handles the bottom
//! half of the interrupt. This design decision was made to mimic the behavior of the
//! original Tock scheduler. In order to ensure fair use of timeslices, when
//! userspace processes are interrupted the systick is paused, and the same process
//! is resumed with the same systick value from when it was interrupted.

use crate::callback::AppId;
use crate::capabilities;
use crate::common::dynamic_deferred_call::DynamicDeferredCall;
use crate::common::list::{List, ListLink, ListNode};
use crate::ipc;
use crate::platform::systick::SysTick;
use crate::platform::{Chip, Platform};
use crate::sched::{Kernel, Scheduler, StoppedExecutingReason};
use core::cell::Cell;

/// A node in the linked list the scheduler uses to track processes
pub struct RoundRobinProcessNode<'a> {
    appid: AppId,
    next: ListLink<'a, RoundRobinProcessNode<'a>>,
}

impl<'a> RoundRobinProcessNode<'a> {
    pub fn new(appid: AppId) -> RoundRobinProcessNode<'a> {
        RoundRobinProcessNode {
            appid,
            next: ListLink::empty(),
        }
    }
}

impl<'a> ListNode<'a, RoundRobinProcessNode<'a>> for RoundRobinProcessNode<'a> {
    fn next(&'a self) -> &'a ListLink<'a, RoundRobinProcessNode> {
        &self.next
    }
}

/// Round Robin Scheduler
pub struct RoundRobinSched<'a> {
    kernel: &'static Kernel,
    time_remaining: Cell<u32>,
    pub processes: List<'a, RoundRobinProcessNode<'a>>,
}

impl<'a> RoundRobinSched<'a> {
    /// How long a process can run before being pre-empted
    const DEFAULT_TIMESLICE_US: u32 = 10000;
    pub const fn new(kernel: &'static Kernel) -> RoundRobinSched<'a> {
        RoundRobinSched {
            kernel,
            time_remaining: Cell::new(0),
            processes: List::new(),
        }
    }
}

impl<'a> Scheduler for RoundRobinSched<'a> {
    /// Main loop.
    fn kernel_loop<P: Platform, C: Chip>(
        &self,
        platform: &P,
        chip: &C,
        ipc: Option<&ipc::IPC>,
        _capability: &dyn capabilities::MainLoopCapability,
    ) -> ! {
        assert!(!chip.systick().dummy());
        let mut reschedule = false;
        loop {
            unsafe {
                chip.service_pending_interrupts();
                DynamicDeferredCall::call_global_instance_while(|| !chip.has_pending_interrupts());

                loop {
                    if chip.has_pending_interrupts()
                        || DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
                        || self.kernel.processes_blocked()
                    {
                        break;
                    }
                    let next = self.processes.head().unwrap().appid;
                    let last_rescheduled = reschedule;
                    reschedule = false;
                    self.kernel.process_map_or((), next, |process| {
                        let timeslice = if last_rescheduled {
                            self.time_remaining.get()
                        } else {
                            Self::DEFAULT_TIMESLICE_US
                        };

                        let (stopped_reason, time_used) = self.kernel.do_process(
                            platform,
                            chip,
                            chip.systick(),
                            process,
                            ipc,
                            Some(timeslice),
                            true,
                        );
                        self.time_remaining
                            .set(self.time_remaining.get() - time_used);
                        reschedule = match stopped_reason {
                            StoppedExecutingReason::KernelPreemption => true,
                            _ => false,
                        }
                    });
                    if !reschedule {
                        self.processes.push_tail(self.processes.pop_head().unwrap());
                    }
                }

                chip.atomic(|| {
                    if !chip.has_pending_interrupts()
                        && !DynamicDeferredCall::global_instance_calls_pending().unwrap_or(false)
                        && self.kernel.processes_blocked()
                    {
                        chip.sleep();
                    }
                });
            };
        }
    }
}
