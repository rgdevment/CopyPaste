use std::ffi::c_void;

const PROC_PIDTBSDINFO: i32 = 3;
const BSD_INFO_SIZE: usize = 136;
const PPID_AT: usize = 16;
const TTY_AT: usize = 108;
const NO_DEVICE: u32 = u32::MAX;
const MOST_CHILDREN: usize = 512;

unsafe extern "C" {
    fn proc_pidinfo(pid: i32, flavor: i32, arg: u64, buffer: *mut c_void, size: i32) -> i32;
    fn proc_listchildpids(ppid: i32, buffer: *mut c_void, size: i32) -> i32;
}

fn bsd_info(pid: i32) -> Option<[u8; BSD_INFO_SIZE]> {
    let mut info = [0u8; BSD_INFO_SIZE];
    let written = unsafe {
        proc_pidinfo(
            pid,
            PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast(),
            BSD_INFO_SIZE as i32,
        )
    };
    (written as usize == BSD_INFO_SIZE).then_some(info)
}

fn word_at(info: &[u8; BSD_INFO_SIZE], at: usize) -> u32 {
    u32::from_ne_bytes([info[at], info[at + 1], info[at + 2], info[at + 3]])
}

pub fn parent_of(pid: i32) -> Option<i32> {
    bsd_info(pid).map(|info| word_at(&info, PPID_AT) as i32)
}

pub fn has_a_terminal(pid: i32) -> bool {
    bsd_info(pid).is_some_and(|info| {
        let device = word_at(&info, TTY_AT);
        device != NO_DEVICE && device != 0
    })
}

pub fn children_of(pid: i32) -> Vec<i32> {
    if pid <= 0 {
        return Vec::new();
    }
    let mut found = vec![0i32; MOST_CHILDREN];
    let counted = unsafe {
        proc_listchildpids(
            pid,
            found.as_mut_ptr().cast(),
            (MOST_CHILDREN * size_of::<i32>()) as i32,
        )
    };
    let Ok(counted) = usize::try_from(counted) else {
        return Vec::new();
    };
    found.truncate(counted.min(MOST_CHILDREN));
    found.retain(|child| *child > 0);
    found
}

pub fn hosts_a_terminal(pid: i32) -> bool {
    if pid <= 0 || has_a_terminal(pid) {
        return false;
    }
    children_of(pid)
        .into_iter()
        .any(|child| has_a_terminal(child) || children_of(child).into_iter().any(has_a_terminal))
}

#[cfg(test)]
#[path = "processes_test.rs"]
mod tests;
