//! Cooperative Scheduler for Tock
//! When hardware interrupts occur while a userspace process is executing,
//! this scheduler executes the top half of the interrupt,
//! and then stops executing the userspace process immediately and handles the bottom
//! half of the interrupt. However it then continues executing the same userspace process
//! that was executing. This scheduler overwrites the systick

use crate::callback::AppId;
use crate::capabilities;
use crate::common::dynamic_deferred_call::DynamicDeferredCall;
use crate::common::list::{List, ListLink, ListNode};
use crate::ipc;
use crate::platform::{Chip, Platform};
use crate::sched::{Kernel, Scheduler, StoppedExecutingReason};

/// A node in the linked list the scheduler uses to track processes
pub struct CoopProcessNode<'a> {
    appid: AppId,
    next: ListLink<'a, CoopProcessNode<'a>>,
}

impl<'a> CoopProcessNode<'a> {
    pub fn new(appid: AppId) -> CoopProcessNode<'a> {
        CoopProcessNode {
            appid,
            next: ListLink::empty(),
        }
    }
}

impl<'a> ListNode<'a, CoopProcessNode<'a>> for CoopProcessNode<'a> {
    fn next(&'a self) -> &'a ListLink<'a, CoopProcessNode> {
        &self.next
    }
}

/// Cooperative Scheduler
pub struct CooperativeSched<'a> {
    kernel: &'static Kernel,
    pub processes: List<'a, CoopProcessNode<'a>>,
}

impl<'a> CooperativeSched<'a> {
    pub const fn new(kernel: &'static Kernel) -> CooperativeSched<'a> {
        CooperativeSched {
            kernel,
            processes: List::new(),
        }
    }
}

impl<'a> Scheduler for CooperativeSched<'a> {
    /// Main loop.
    fn kernel_loop<P: Platform, C: Chip>(
        &mut self,
        platform: &P,
        chip: &C,
        ipc: Option<&ipc::IPC>,
        _capability: &dyn capabilities::MainLoopCapability,
    ) -> ! {
        let mut reschedule;
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
                    reschedule = false;
                    self.kernel.process_map_or((), next, |process| {
                        reschedule = match self
                            .kernel
                            .do_process(platform, chip, &(), process, ipc, None, true)
                            .0
                        {
                            StoppedExecutingReason::KernelPreemption => true,
                            _ => false,
                        };
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
